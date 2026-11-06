//! Folder edits retain their original identity/revision and patch only visible fields.
use super::*;

#[derive(Clone, PartialEq, Eq)]
pub(super) struct FolderTarget {
    revision: String,
    identity: String,
    generation: u64,
}
pub(super) struct FolderEditor {
    revision: String,
    target: Option<FolderTarget>,
    name: Entity<InputState>,
    local: Entity<InputState>,
    remote: Entity<InputState>,
}
impl SyncView {
    pub(super) fn selected_folder(&self) -> Option<&SyncFolderRow> {
        let SyncSnapshot::Present(local) = self.snapshot.as_ref()? else {
            return None;
        };
        local.folders.get(self.selected?)
    }
    fn folder_target(&self) -> Option<FolderTarget> {
        Some(FolderTarget {
            revision: self.revision()?,
            identity: self.selected_folder()?.identity.clone()?,
            generation: self.generation,
        })
    }
    fn target_current(&self, target: &FolderTarget) -> bool {
        self.folder_target().as_ref() == Some(target)
    }
    fn edit_folder(&mut self, existing: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy || self.operation.busy || self.editor.is_some() {
            return;
        }
        let Some(revision) = self.revision() else {
            return;
        };
        let target = if existing {
            let Some(target) = self.folder_target() else {
                return;
            };
            Some(target)
        } else {
            None
        };
        if !matches!(self.snapshot, Some(SyncSnapshot::Present(_))) {
            return;
        }
        self.confirmation = None;
        self.folder_delete = None;
        self.asset_editor = None;
        self.folder_editor = Some(FolderEditor {
            revision,
            target,
            name: cx.new(|cx| InputState::new(window, cx)),
            local: cx.new(|cx| InputState::new(window, cx)),
            remote: cx.new(|cx| InputState::new(window, cx)),
        });
        cx.notify();
    }
    fn save_folder(&mut self, cx: &mut Context<Self>) {
        let Some(editor) = &self.folder_editor else {
            return;
        };
        if self.revision().as_ref() != Some(&editor.revision)
            || editor
                .target
                .as_ref()
                .is_some_and(|target| !self.target_current(target))
        {
            return;
        }
        let value = |field: &Entity<InputState>| {
            let value = field.read(cx).value().to_string();
            (!value.trim().is_empty()).then(|| value.trim().to_string())
        };
        let patch = FolderPatch {
            name: value(&editor.name),
            local_path: value(&editor.local),
            remote_path: value(&editor.remote),
        };
        let change = if let Some(target) = &editor.target {
            FolderChange::Edit {
                identity: target.identity.clone(),
                patch,
            }
        } else {
            FolderChange::Add(patch)
        };
        self.start(
            Operation::Folder {
                revision: editor.revision.clone(),
                change,
            },
            cx,
        );
    }
    fn delete_folder(&mut self, target: &FolderTarget, cx: &mut Context<Self>) {
        if self.folder_delete.as_ref() != Some(target) || !self.target_current(target) {
            return;
        }
        self.start(
            Operation::Folder {
                revision: target.revision.clone(),
                change: FolderChange::Delete {
                    identity: target.identity.clone(),
                },
            },
            cx,
        );
    }
    fn enable_folder(&mut self, target: &FolderTarget, enabled: bool, cx: &mut Context<Self>) {
        if !self.target_current(target) || self.folder_editor.is_some() || self.editor.is_some() {
            return;
        }
        self.start(
            Operation::Folder {
                revision: target.revision.clone(),
                change: FolderChange::Enabled {
                    identity: target.identity.clone(),
                    enabled,
                },
            },
            cx,
        );
    }
    pub(super) fn render_folder_controls(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let locked = self.busy || self.operation.busy;
        let mut section = div().flex().flex_col().gap(px(theme::CONTROL_GAP));
        if matches!(self.snapshot, Some(SyncSnapshot::Absent)) {
            return section.child(text(
                cx,
                "Save WebDAV settings before adding sync folders.",
                "请先保存 WebDAV 设置，再添加同步文件夹。",
            ));
        }
        let legacy = matches!(&self.snapshot,Some(SyncSnapshot::Present(local)) if local.folders.iter().any(|folder|folder.identity.is_none()));
        if legacy {
            return section.child(text(
                cx,
                SyncError::LegacyFolders.message(),
                "旧格式文件夹条目只读，请通过 CCR 迁移为规范 folders 格式后再编辑。",
            ));
        }
        section = section.child(semantics::button(
            Button::new("sync-folder-add")
                .label(text(cx, "Add sync folder", "添加同步文件夹"))
                .ghost()
                .on_click(cx.listener(|this, _, window, cx| this.edit_folder(false, window, cx))),
            locked || self.editor.is_some() || self.snapshot.is_none(),
            false,
            cx,
        ));
        if let Some(target) = self.folder_target() {
            let enabled = self.selected_folder().is_some_and(|folder| folder.enabled);
            let toggle = target.clone();
            let delete = target.clone();
            section = section.child(
                div()
                    .flex()
                    .gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(
                        Button::new("sync-folder-edit")
                            .label(text(cx, "Edit selected folder", "编辑所选文件夹"))
                            .ghost()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.edit_folder(true, window, cx)
                            })),
                        locked || self.editor.is_some(),
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("sync-folder-enabled")
                            .label(if enabled {
                                text(cx, "Disable selected folder", "停用所选文件夹")
                            } else {
                                text(cx, "Enable selected folder", "启用所选文件夹")
                            })
                            .ghost()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.enable_folder(&toggle, !enabled, cx)
                            })),
                        locked || self.editor.is_some() || self.folder_editor.is_some(),
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("sync-folder-delete")
                            .label(text(cx, "Delete selected folder", "删除所选文件夹"))
                            .ghost()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if this.target_current(&delete) {
                                    this.folder_delete = Some(delete.clone());
                                    this.confirmation = None;
                                    cx.notify();
                                }
                            })),
                        locked || self.editor.is_some() || self.folder_editor.is_some(),
                        false,
                        cx,
                    )),
            );
        }
        if let Some(editor) = &self.folder_editor {
            section=section.child(div().id("sync-folder-editor").test_support().flex().flex_col().gap(px(theme::CONTROL_GAP))
                .child(if editor.target.is_some(){text(cx,"Blank fields keep their saved values. Existing exclusions and automatic-sync settings are preserved.","字段留空保留原值；已有排除规则和自动同步设置不变。")}else{text(cx,"Enter a unique name, local file or directory, and remote path. New folders are enabled; automatic sync remains off.","填写唯一名称、本地文件或目录及远程路径。新文件夹默认启用，自动同步关闭。")})
                .child(text(cx,"Folder name","文件夹名称")).child(Input::new(&editor.name).accessibility_id("sync-folder-name").aria_label(text(cx,"Folder name","文件夹名称")))
                .child(text(cx,"Local path","本地路径")).child(Input::new(&editor.local).accessibility_id("sync-folder-local").aria_label(text(cx,"Local path","本地路径")))
                .child(text(cx,"Remote path","远程路径")).child(Input::new(&editor.remote).accessibility_id("sync-folder-remote").aria_label(text(cx,"Remote path","远程路径")))
                .child(div().flex().gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(Button::new("sync-folder-save").label(text(cx,"Save folder","保存文件夹")).on_click(cx.listener(|this,_,_,cx|this.save_folder(cx))),locked,false,cx))
                    .child(semantics::button(Button::new("sync-folder-cancel").label(text(cx,"Cancel editing","取消编辑")).ghost().on_click(cx.listener(|this,_,_,cx|{this.folder_editor=None;cx.notify();})),locked,false,cx))));
        }
        if let Some(target) = self.folder_delete.clone() {
            section=section.child(div().id("sync-folder-delete-confirmation").test_support().flex().flex_col().gap(px(theme::CONTROL_GAP))
                .child(text(cx,"Remove this folder from sync settings? Local and remote files will remain untouched.","从同步设置中移除此文件夹？本地与远程文件均保持不变。"))
                .child(semantics::button(Button::new("sync-folder-delete-cancel").label(text(cx,"Cancel","取消")).ghost().on_click(cx.listener(|this,_,_,cx|{this.folder_delete=None;cx.notify();})),locked,false,cx))
                .child(semantics::button(Button::new("sync-folder-delete-confirm").label(text(cx,"Confirm removal","确认移除")).on_click(cx.listener(move|this,_,_,cx|this.delete_folder(&target,cx))),locked,false,cx)));
        }
        section
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_app::runtime::RuntimeOwner;
    use gpui_kit::test::{TestAppContextExt, TestWindowExt};
    use gpui_kit::{TestAppContext, WindowOptions};
    use std::time::Duration;

    #[gpui_kit::test]
    async fn folder_editor_and_delete_confirmation_keep_the_original_target(
        cx: &mut TestAppContext,
    ) {
        cx.executor().allow_parking();
        let root = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::new().unwrap();
        let mut config = ccr_sync::SyncFoldersConfig::default();
        config.webdav.url = "https://dav.example.test/".into();
        config.webdav.username = "fixture".into();
        for name in ["first", "second"] {
            config.folders.push(
                ccr_sync::SyncFolder::builder()
                    .name(name)
                    .local_path(root.path().join(name).to_string_lossy().into_owned())
                    .remote_path(format!("/{name}"))
                    .build()
                    .unwrap(),
            );
        }
        let path = root.path().join("sync_folders.toml");
        std::fs::write(&path, toml::to_string(&config).unwrap()).unwrap();
        let services = Services::new(
            owner.handle(),
            crate::runtime::ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("registry.toml"),
                delay: Duration::ZERO,
            },
        );
        let retained = services.clone();
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            cx.set_global(crate::locales::Language::English);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| SyncView::new(services, window, cx))
            })
            .unwrap()
        });
        view.update(cx, |this, cx| this.refresh(cx));
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx).snapshot.is_some()
        })
        .await;
        let stale = cx
            .update_window(handle, |_, window, cx| {
                window.render_frame(cx);
                view.update(cx, |this, cx| {
                    this.selected = Some(0);
                    this.edit_folder(true, window, cx);
                    let target = this.folder_target().unwrap();
                    this.folder_delete = Some(target.clone());
                    this.selected = Some(1);
                    this.delete_folder(&target, cx);
                    this.save_folder(cx);
                    assert!(!retained.sync_operation_state().busy);
                    target
                })
            })
            .unwrap();
        let before = std::fs::read(&path).unwrap();
        view.update(cx, |this, cx| {
            this.refresh(cx);
            this.delete_folder(&stale, cx);
            assert!(this.folder_delete.is_none());
            assert!(this.folder_editor.is_none());
        });
        assert_eq!(std::fs::read(&path).unwrap(), before);
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| !view.read(cx).busy)
            .await;
        cx.update_window(handle, |_, window, cx| {
            view.update(cx, |this, cx| {
                this.selected = Some(0);
                this.edit_folder(true, window, cx);
                let name = this.folder_editor.as_ref().unwrap().name.clone();
                name.update(cx, |state, cx| state.set_value("renamed", window, cx));
                this.save_folder(cx);
            });
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            !retained.sync_operation_state().busy
                && view.read(cx).folder_editor.is_none()
                && !view.read(cx).busy
        })
        .await;
        let saved: ccr_sync::SyncFoldersConfig =
            toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(saved.folders[0].name, "renamed");
        assert_eq!(saved.folders[1].name, "second");
        owner.shutdown();
    }
}
