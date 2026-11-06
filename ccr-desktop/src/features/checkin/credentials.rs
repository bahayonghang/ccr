//! A read belongs to one editor instance, never the app's retained job state.
use super::*;
use crate::runtime::checkin::{AccountCredentials, OperationError};

#[derive(Default)]
pub(super) struct CredentialRead {
    pub loading: bool,
    pub failed: Option<OperationError>,
    pub advanced: bool,
    request: u64,
    loaded: Option<AccountCredentials>,
    task: Option<Task<()>>,
}

impl operations::Editor {
    pub(super) fn cookie_patch(
        &self,
        cx: &App,
    ) -> Result<Option<ccr_core::Secret>, crate::runtime::checkin::OperationError> {
        let value = self.fields[if self.credentials.advanced { 1 } else { 3 }]
            .read(cx)
            .value();
        if value.is_empty() {
            return Ok(None);
        }
        if self.credentials.advanced {
            return Ok(Some(value.to_string().into()));
        }
        let original = self.fields[1].read(cx).value();
        let mut cookies: serde_json::Value = serde_json::from_str(if original.is_empty() {
            "{}"
        } else {
            original.as_str()
        })
        .map_err(|_| crate::runtime::checkin::OperationError::Invalid)?;
        cookies
            .as_object_mut()
            .ok_or(crate::runtime::checkin::OperationError::Invalid)?
            .insert(
                "session".into(),
                serde_json::Value::String(value.to_string()),
            );
        Ok(Some(cookies.to_string().into()))
    }
}

impl CheckinView {
    pub(super) fn advanced_cookies(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(editor) = &mut self.editor else {
            return;
        };
        if self.operation.busy || editor.credentials.loading || editor.credentials.advanced {
            return;
        }
        match editor.cookie_patch(cx) {
            Ok(cookies) => {
                if let Some(cookies) = cookies {
                    editor.fields[1].update(cx, |state, cx| {
                        state.set_value(cookies.expose().to_owned(), window, cx)
                    });
                }
                editor.credentials.advanced = true;
            }
            Err(error) => {
                let mut state = (*self.operation).clone();
                state.result = Some(Err(error));
                self.operation = Arc::new(state);
            }
        }
        cx.notify();
    }
    pub(super) fn load_account_credentials(&mut self, cx: &mut Context<Self>) {
        let Some(editor) = &mut self.editor else {
            return;
        };
        let operations::EditorKind::Account(Some(id)) = &editor.kind else {
            return;
        };
        let id = id.clone();
        self.generation += 1;
        let request = self.generation;
        editor.credentials.request = request;
        editor.credentials.loading = true;
        editor.credentials.failed = None;
        let key = RequestKey {
            environment: EnvironmentId::Local(
                if self.services.is_fixture() {
                    "fixture"
                } else {
                    "local"
                }
                .into(),
            ),
            platform: Platform::Claude,
            generation: request,
            request_id: request,
        };
        match self.services.checkin_account_credentials(key, id.clone()) {
            Ok(mut subscription) => {
                editor.credentials.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                this.accept_account_credentials(request, &id, result.as_ref());
                                cx.notify();
                            });
                            break;
                        }
                    }
                }));
            }
            Err(_) => {
                editor.credentials.loading = false;
                editor.credentials.failed = Some(OperationError::Interrupted);
            }
        }
    }

    fn accept_account_credentials(
        &mut self,
        request: u64,
        id: &str,
        result: &Completion<AccountCredentials, crate::runtime::checkin::OperationError>,
    ) {
        let Some(editor) = &mut self.editor else {
            return;
        };
        if editor.credentials.request != request
            || !matches!(&editor.kind,
            operations::EditorKind::Account(Some(current)) if current == id)
        {
            return;
        }
        editor.credentials.loading = false;
        match result {
            Completion::Success(value) if value.account_id == id => {
                editor.credentials.failed = None;
                editor.credentials.loaded = Some(value.clone());
            }
            Completion::Failed(error) | Completion::CleanupFailed(error) => {
                editor.credentials.failed = Some(*error)
            }
            _ => editor.credentials.failed = Some(OperationError::Interrupted),
        }
    }

    pub(super) fn fill_account_credentials(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(editor) = &mut self.editor else {
            return;
        };
        let Some(value) = editor.credentials.loaded.take() else {
            return;
        };
        for (field, value) in editor.fields.iter().zip([
            value.name.as_str(),
            value.cookies.expose(),
            value.api_user.expose(),
        ]) {
            field.update(cx, |state, cx| {
                state.set_value(value.to_owned(), window, cx)
            });
        }
        let session = serde_json::from_str::<serde_json::Value>(value.cookies.expose())
            .ok()
            .and_then(|cookies| {
                cookies
                    .get("session")
                    .and_then(|value| value.as_str())
                    .map(str::to_owned)
            })
            .unwrap_or_default();
        editor.fields[3].update(cx, |state, cx| state.set_value(session, window, cx));
    }
}

pub(super) fn read_error_notice(error: OperationError, cx: &App) -> gpui_kit::AnyElement {
    let message = if error == OperationError::CryptoKeyUnavailable {
        text(
            cx,
            "The original credential key is missing or unreadable. Restore it before using saved credentials. Name and enabled status can still be edited.",
            "原凭据密钥缺失或无法读取，请先恢复该密钥再使用已保存凭据。仍可编辑名称和启用状态。",
        )
    } else {
        text(
            cx,
            "Saved credentials could not be read. Blank fields retain them; enter replacements to update.",
            "无法读取已保存凭据。留空会保留原值，可输入替换值进行更新。",
        )
    };
    div()
        .id("checkin-credential-read-error")
        .test_support()
        .role(gpui_kit::Role::Status)
        .aria_label(message)
        .child(message)
        .into_any_element()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::runtime::{
        ConfigSource,
        checkin::{Operation, Outcome},
    };
    use ccr_app::runtime::RuntimeOwner;
    use ccr_db::models::checkin::{CreateAccountRequest, CreateProviderRequest};
    use gpui_kit::test::{TestAppContextExt, TestWindowExt};
    use gpui_kit::{AppContext, TestAppContext, WindowOptions};
    use std::time::{Duration, Instant};

    fn run(services: &Services, operation: Operation, n: u64) {
        services
            .start_checkin(
                RequestKey {
                    environment: EnvironmentId::Local("fixture".into()),
                    platform: Platform::Claude,
                    generation: n,
                    request_id: n,
                },
                operation,
            )
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let state = services.checkin_operation_state();
            if !state.busy {
                assert_eq!(state.result, Some(Ok(Outcome::Saved)));
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[gpui_kit::test]
    async fn missing_key_editor_requires_recovery_but_keeps_metadata_savable(
        cx: &mut TestAppContext,
    ) {
        cx.executor().allow_parking();
        let root = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::new().unwrap();
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("registry.toml"),
                delay: Duration::ZERO,
            },
        );
        run(
            &services,
            Operation::AddProvider(CreateProviderRequest {
                name: "fixture".into(),
                base_url: "https://example.invalid".into(),
                checkin_path: None,
                balance_path: None,
                user_info_path: None,
                auth_header: None,
                auth_prefix: None,
                builtin_id: None,
            }),
            1,
        );
        let provider = services
            .checkin_operation_state()
            .snapshot
            .as_ref()
            .unwrap()
            .providers[0]
            .id
            .clone();
        run(
            &services,
            Operation::AddAccount(CreateAccountRequest {
                provider_id: provider,
                name: "original".into(),
                cookies_json: r#"{"session":"original-secret","other":"keep"}"#.into(),
                api_user: "7".into(),
                extra_config: "{}".into(),
            }),
            2,
        );
        let id = services
            .checkin_operation_state()
            .snapshot
            .as_ref()
            .unwrap()
            .accounts[0]
            .id
            .clone();
        let pool = ccr_db::database::create_pool(&root.path().join("checkin.db"), None).unwrap();
        let accounts = ccr_checkin::AccountManager::with_db(
            &root.path().join("checkin-crypto"),
            ccr_db::database::DbAccess::Pool(pool),
        );
        let encrypted = accounts.get(&id).unwrap().cookies_json_encrypted;
        let key = root.path().join("checkin-crypto/crypto.key");
        std::fs::remove_file(&key).unwrap();
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| CheckinView::new(services, window, cx))
            })
            .unwrap()
        });
        cx.update_window(handle, |_, window, cx| {
            view.update(cx, |this, cx| {
                this.edit(
                    operations::EditorKind::Account(Some(id.clone())),
                    window,
                    cx,
                )
            });
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !view.read(cx).editor.as_ref().unwrap().credentials.loading
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert_eq!(
                view.read(cx).editor.as_ref().unwrap().credentials.failed,
                Some(OperationError::CryptoKeyUnavailable)
            );
            let warning = window.find("checkin-credential-read-error");
            assert!(warning.label().unwrap().contains("Restore it"));
            assert!(!warning.label().unwrap().contains("enter replacements"));
            let session = view.read(cx).editor.as_ref().unwrap().fields[3].clone();
            session.update(cx, |state, cx| {
                state.set_value("must-not-replace", window, cx)
            });
            window.click("checkin-save", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !view.read(cx).operation.busy
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            assert_eq!(
                view.read(cx).operation.result,
                Some(Err(OperationError::CryptoKeyUnavailable))
            );
            let editor = view.read(cx).editor.as_ref().unwrap();
            let session = editor.fields[3].clone();
            let name = editor.fields[0].clone();
            session.update(cx, |state, cx| state.set_value("", window, cx));
            name.update(cx, |state, cx| {
                state.set_value("metadata-without-key", window, cx)
            });
            window.render_frame(cx);
            window.click("checkin-save", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).editor.is_none()
        })
        .await;
        assert!(!key.exists());
        let account = accounts.get(&id).unwrap();
        assert_eq!(account.name, "metadata-without-key");
        assert_eq!(account.cookies_json_encrypted, encrypted);
        cx.update_window(handle, |_, window, cx| {
            view.update(cx, |this, cx| this.suspend(cx));
            window.remove_window();
        })
        .unwrap();
        owner.shutdown();
    }

    #[gpui_kit::test]
    async fn selected_credentials_fill_masked_editor_and_stale_results_are_discarded(
        cx: &mut TestAppContext,
    ) {
        cx.executor().allow_parking();
        let root = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::new().unwrap();
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("registry.toml"),
                delay: Duration::ZERO,
            },
        );
        run(
            &services,
            Operation::AddProvider(CreateProviderRequest {
                name: "fixture".into(),
                base_url: "https://example.invalid".into(),
                checkin_path: None,
                balance_path: None,
                user_info_path: None,
                auth_header: None,
                auth_prefix: None,
                builtin_id: None,
            }),
            1,
        );
        let provider = services
            .checkin_operation_state()
            .snapshot
            .as_ref()
            .unwrap()
            .providers[0]
            .id
            .clone();
        for (n, name, cookie) in [(2, "first", "first-secret"), (3, "second", "second-secret")] {
            run(
                &services,
                Operation::AddAccount(CreateAccountRequest {
                    provider_id: provider.clone(),
                    name: name.into(),
                    cookies_json: format!(r#"{{"session":"{cookie}","other":"opaque-extra"}}"#)
                        .into(),
                    api_user: format!("{name}-user"),
                    extra_config: "{}".into(),
                }),
                n,
            );
        }
        let accounts = services
            .checkin_operation_state()
            .snapshot
            .as_ref()
            .unwrap()
            .accounts
            .clone();
        let first = accounts
            .iter()
            .find(|a| a.name == "first")
            .unwrap()
            .id
            .clone();
        let second = accounts
            .iter()
            .find(|a| a.name == "second")
            .unwrap()
            .id
            .clone();
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| CheckinView::new(services, window, cx))
            })
            .unwrap()
        });
        let stale_request = cx
            .update_window(handle, |_, window, cx| {
                view.update(cx, |this, cx| {
                    this.edit(
                        operations::EditorKind::Account(Some(first.clone())),
                        window,
                        cx,
                    );
                    let request = this.editor.as_ref().unwrap().credentials.request;
                    assert!(this.editor.as_ref().unwrap().credentials.loading);
                    this.edit(
                        operations::EditorKind::Account(Some(second.clone())),
                        window,
                        cx,
                    );
                    request
                })
            })
            .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !view.read(cx).editor.as_ref().unwrap().credentials.loading
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            let editor = view.read(cx).editor.as_ref().unwrap();
            assert_eq!(editor.fields[0].read(cx).value().as_str(), "second");
            assert_eq!(
                editor.fields[1].read(cx).value().as_str(),
                r#"{"session":"second-secret","other":"opaque-extra"}"#
            );
            assert_eq!(editor.fields[2].read(cx).value().as_str(), "second-user");
            assert_eq!(editor.fields[3].read(cx).value().as_str(), "second-secret");
            let session_input = editor.fields[3].clone();
            for node in gpui_kit::base::test_support::snapshots(window) {
                for visible in [node.label(), node.value()].into_iter().flatten() {
                    assert!(
                        !visible.contains("first-secret")
                            && !visible.contains("second-secret")
                            && !visible.contains("second-user")
                    );
                }
            }
            session_input.update(cx, |state, cx| {
                state.set_value("edited-session", window, cx)
            });
            window.render_frame(cx);
            assert!(
                gpui_kit::base::test_support::snapshots(window)
                    .iter()
                    .any(|node| node.label() == Some("Session cookie"))
            );
            assert!(
                !gpui_kit::base::test_support::snapshots(window)
                    .iter()
                    .any(|node| node.label() == Some("Cookies JSON"))
            );
            window.click("checkin-save", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).editor.is_none()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            view.update(cx, |this, cx| {
                this.edit(
                    operations::EditorKind::Account(Some(second.clone())),
                    window,
                    cx,
                )
            });
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !view.read(cx).editor.as_ref().unwrap().credentials.loading
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            let editor = view.read(cx).editor.as_ref().unwrap();
            assert_eq!(editor.fields[3].read(cx).value().as_str(), "edited-session");
            let saved: serde_json::Value =
                serde_json::from_str(&editor.fields[1].read(cx).value()).unwrap();
            assert_eq!(saved["other"], "opaque-extra");
            assert_eq!(saved["session"], "edited-session");
            view.update(cx, |this, cx| this.advanced_cookies(window, cx));
            window.render_frame(cx);
            assert!(
                gpui_kit::base::test_support::snapshots(window)
                    .iter()
                    .any(|node| node.label() == Some("Cookies JSON"))
            );
            view.update(cx, |this, cx| {
                this.accept_account_credentials(
                    stale_request,
                    &first,
                    &Completion::Success(AccountCredentials {
                        account_id: first.clone(),
                        name: "stale".into(),
                        cookies: "stale-secret".into(),
                        api_user: "stale-user".into(),
                    }),
                );
                assert!(this.editor.as_ref().unwrap().credentials.loaded.is_none());
                this.suspend(cx);
                assert!(this.editor.is_none());
                this.accept_account_credentials(
                    stale_request,
                    &first,
                    &Completion::Success(AccountCredentials {
                        account_id: first.clone(),
                        name: "late".into(),
                        cookies: "late-secret".into(),
                        api_user: "late-user".into(),
                    }),
                );
                assert!(this.editor.is_none());
            });
            window.remove_window();
        })
        .unwrap();
        owner.shutdown();
    }
}
