# Native CSP failure diagnosis

Attempt 1 used an incorrect #root selector. The actual mount is #app in ccr-ui/index.html and src/main.tsx. Attempt 2 used incorrect source-button text. The Chinese settingsRaw.sourceTab label is 源文件. Both attempts remain failed harness runs; neither reached editor or CSP acceptance.

Attempts 3 and 4 opened the real native Settings page, required plaintext confirmation, and loaded the synthetic file in CodeMirror. Computed editor display was flex, font size was 13px, and both minimum heights were 448px (28rem at 16px).

Attempt 4 recorded a style-src-elem violation before plaintext confirmation. The source was assets/base-modal-CepZGn4m.js. The added stylesheet contained body[data-scroll-locked] and had no nonce. CodeMirror later inserted a separate style with the page nonce. This identifies the missing nonce in the Radix/react-remove-scroll/react-style-singleton runtime style path. The scroll-lock style is blocked by the production policy.

The fix boundary is shared page-nonce initialization before React mounting. Reuse the page nonce already consumed by CodeMirror. The installed get-nonce package supports the __webpack_nonce__ fallback. No CSP policy relaxation, modal behavior rewrite, dependency addition, or user configuration access is needed.

Native acceptance remains failed until a rebuilt custom-protocol binary passes the same style, script rejection, save, and process cleanup assertions. The running Windows CI retains its frozen input; product changes wait for that run to settle.
