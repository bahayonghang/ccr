# i18n 调用形式约定

视图与外壳统一用这一套。不要引入第二种写法。

## 组件内

```tsx
import { useAppT, useAppLocale, useAppTt } from '@/i18n'

const t = useAppT()
t('views.foo.bar')
t('views.foo.count', { n: 3 })

const locale = useAppLocale()
const tt = useAppTt()
```

`useTranslation()` 的 `t` 也可以，不要传命名空间：禁止 `useTranslation('views')`。

有可选注入时用 `useResolvedT(tProp)`，始终订阅语言切换。

## 组件外（工具函数、非 hook）

```ts
import { translate } from '@/i18n'
translate('errors.network')
translate('common.about.title', { name: 'CCR' })
```

`i18n.t('errors.network')` 等价，优先 `translate`，返回值收成 `string`。

## 禁止

- `<Trans>`
- `useTranslation('views')` 这类命名空间拆分
- `withTranslation` HOC

## 语言切换

`setLocale('en-US')` → `changeLanguage` + 写入 `ccr-ui-locale` + `document.documentElement.lang`。不要自己读 localStorage 做一份平行 catalog。
