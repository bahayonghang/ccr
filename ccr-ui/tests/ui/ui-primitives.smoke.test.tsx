import { readFile } from 'node:fs/promises'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { useState } from 'react'
import { beforeAll, describe, expect, it } from 'vitest'
import {
  Checkbox,
  Combobox,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  Popover,
  PopoverContent,
  PopoverTrigger,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  StatTile,
  Switch,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/ui'

// 9 类 shadcn 原语的消费示例（AC4）。示例放在 smoke 测试内（implement.md 允许），
// 不放在业务代码。每个示例断言一项基本交互。
//
// jsdom 无 ResizeObserver：Radix popper 的 floating-ui autoUpdate 会检测缺失并退化，
// 但 cmdk 无条件 `new ResizeObserver(...)`，需在 jsdom 下打桩。弹层内容经 Portal
// 渲染到 document.body，测试按 role 查询即可。


beforeAll(() => {
  // jsdom 无 ResizeObserver：Radix popper 的 floating-ui autoUpdate 会检测缺失并退化，
  // 但 cmdk 无条件 `new ResizeObserver(...)`，需在 jsdom 下打桩。
  class ResizeObserverStub {
    observe(): void {}
    unobserve(): void {}
    disconnect(): void {}
  }
  if (typeof globalThis.ResizeObserver === 'undefined') {
    globalThis.ResizeObserver = ResizeObserverStub as unknown as typeof ResizeObserver
  }

  // jsdom 未实现 scrollIntoView，Radix Select 聚焦选项时调用它
  if (typeof Element.prototype.scrollIntoView !== 'function') {
    Element.prototype.scrollIntoView = () => {}
  }
})


const textOf = (element: Element | null): string => element?.textContent ?? ''

describe('ui-primitives（08-22-design-system 批次 3，AC4 消费示例）', () => {
  it('Dialog：打开渲染内容，关闭后移除', async () => {
    render(
      <Dialog>
        <DialogTrigger asChild>
          <button type="button">打开弹窗</button>
        </DialogTrigger>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>确认操作</DialogTitle>
            <DialogDescription>这是弹窗正文。</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <DialogClose asChild>
              <button type="button">取消</button>
            </DialogClose>
          </DialogFooter>
        </DialogContent>
      </Dialog>,
    )

    expect(screen.queryByRole('dialog')).toBeNull()
    fireEvent.click(screen.getByRole('button', { name: '打开弹窗' }))
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeNull()
    })
    expect(textOf(screen.queryByRole('heading', { name: '确认操作' }))).toBe('确认操作')
    expect(textOf(screen.queryByText('这是弹窗正文。'))).toBe('这是弹窗正文。')

    fireEvent.click(screen.getByRole('button', { name: '取消' }))
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull()
    })
  })

  it('Popover：触发后渲染浮层内容', async () => {
    render(
      <Popover>
        <PopoverTrigger asChild>
          <button type="button">打开浮层</button>
        </PopoverTrigger>
        <PopoverContent>浮层内容</PopoverContent>
      </Popover>,
    )

    expect(screen.queryByText('浮层内容')).toBeNull()
    fireEvent.click(screen.getByRole('button', { name: '打开浮层' }))
    await waitFor(() => {
      expect(screen.queryByText('浮层内容')).not.toBeNull()
    })
  })

  it('DropdownMenu：菜单项点击触发回调', async () => {
    let picked = ''
    render(
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <button type="button">更多操作</button>
        </DropdownMenuTrigger>
        <DropdownMenuContent>
          <DropdownMenuLabel>操作</DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuItem onSelect={() => { picked = '编辑' }}>编辑</DropdownMenuItem>
          <DropdownMenuItem onSelect={() => { picked = '删除' }}>删除</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>,
    )

    fireEvent.pointerDown(screen.getByRole('button', { name: '更多操作' }), {
      button: 0,
      ctrlKey: false,
    })
    await waitFor(() => {
      expect(screen.queryByRole('menu')).not.toBeNull()
    })
    fireEvent.click(screen.getByRole('menuitem', { name: '删除' }))
    await waitFor(() => {
      expect(picked).toBe('删除')
    })
  })

  it('Tooltip：hover 触发显示提示', async () => {
    render(
      <TooltipProvider>
        <Tooltip delayDuration={0}>
          <TooltipTrigger asChild>
            <button type="button">悬停我</button>
          </TooltipTrigger>
          <TooltipContent>这是提示</TooltipContent>
        </Tooltip>
      </TooltipProvider>,
    )

    expect(screen.queryByRole('tooltip')).toBeNull()
    fireEvent.pointerMove(screen.getByRole('button', { name: '悬停我' }))
    await waitFor(() => {
      expect(screen.queryByRole('tooltip')).not.toBeNull()
    })
    expect(textOf(screen.queryByText('这是提示'))).toBe('这是提示')
  })

  it('Tabs：切换 tab 后内容切换', async () => {
    render(
      <Tabs defaultValue="account">
        <TabsList>
          <TabsTrigger value="account">账号</TabsTrigger>
          <TabsTrigger value="password">密码</TabsTrigger>
        </TabsList>
        <TabsContent value="account">账号设置</TabsContent>
        <TabsContent value="password">密码设置</TabsContent>
      </Tabs>,
    )

    expect(textOf(screen.queryByText('账号设置'))).toBe('账号设置')
    // Radix 对未激活内容用 hidden 属性隐藏，testing-library 查询视为不可达
    expect(screen.queryByText('密码设置')).toBeNull()
    // Radix Tabs 的切换事件是 onMouseDown（与浏览器原生 tab 行为一致），fireEvent.click
    // 不触发切换
    fireEvent.mouseDown(screen.getByRole('tab', { name: '密码' }), { button: 0 })
    await waitFor(() => {
      expect(textOf(screen.queryByText('密码设置'))).toBe('密码设置')
    })
    expect(screen.queryByText('账号设置')).toBeNull()
    // Radix 键盘导航/激活态：data-state 由 Radix 管理
    expect(screen.getByRole('tab', { name: '密码' }).getAttribute('data-state')).toBe('active')
  })

  it('Combobox：输入过滤选项', async () => {
    render(
      <Combobox>
        <ComboboxInput placeholder="搜索平台…" />
        <ComboboxList>
          <ComboboxItem value="claude">Claude</ComboboxItem>
          <ComboboxItem value="codex">Codex</ComboboxItem>
          <ComboboxItem value="gemini">Gemini</ComboboxItem>
          <ComboboxEmpty>无匹配项</ComboboxEmpty>
        </ComboboxList>
      </Combobox>,
    )

    expect(textOf(screen.queryByText('Claude'))).toBe('Claude')
    fireEvent.change(screen.getByPlaceholderText('搜索平台…'), { target: { value: 'cod' } })
    await waitFor(() => {
      expect(textOf(screen.queryByText('Codex'))).toBe('Codex')
    })
    expect(screen.queryByText('Claude')).toBeNull()
    expect(screen.queryByText('Gemini')).toBeNull()
  })

  it('Select：选择后更新值', async () => {
    function SelectExample() {
      const [value, setValue] = useState('codex')
      return (
        <Select value={value} onValueChange={setValue}>
          <SelectTrigger aria-label="平台">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="claude">Claude</SelectItem>
            <SelectItem value="codex">Codex</SelectItem>
          </SelectContent>
        </Select>
      )
    }

    render(<SelectExample />)
    expect(textOf(screen.getByRole('combobox'))).toBe('Codex')
    fireEvent.click(screen.getByRole('combobox'))
    await waitFor(() => {
      expect(screen.queryByRole('listbox')).not.toBeNull()
    })
    fireEvent.click(screen.getByRole('option', { name: 'Claude' }))
    await waitFor(() => {
      expect(textOf(screen.getByRole('combobox'))).toBe('Claude')
    })
  })

  it('Switch：点击切换选中态', async () => {
    function SwitchExample() {
      const [checked, setChecked] = useState(false)
      return <Switch checked={checked} onCheckedChange={setChecked} aria-label="通知" />
    }

    render(<SwitchExample />)
    const switcher = screen.getByRole('switch', { name: '通知' })
    expect(switcher.getAttribute('data-state')).toBe('unchecked')
    fireEvent.click(switcher)
    await waitFor(() => {
      expect(switcher.getAttribute('data-state')).toBe('checked')
    })
  })

  it('Checkbox：点击切换勾选态', async () => {
    function CheckboxExample() {
      const [checked, setChecked] = useState<boolean>(false)
      return (
        <label>
          <Checkbox
            checked={checked}
            onCheckedChange={(next) => setChecked(next === true)}
            aria-label="记住我"
          />
          记住我
        </label>
      )
    }

    render(<CheckboxExample />)
    const checkbox = screen.getByRole('checkbox', { name: '记住我' })
    expect(checkbox.getAttribute('data-state')).toBe('unchecked')
    fireEvent.click(checkbox)
    await waitFor(() => {
      expect(checkbox.getAttribute('data-state')).toBe('checked')
    })
  })

  it('StatTile：无 tone 时为裸砖，不带 data-tone', () => {
    const { container } = render(<StatTile label="就绪" value="3" hint="项" />)
    const value = container.querySelector('.stat-tile__value')
    expect(value?.classList.contains('stat-tile__value--badge')).toBe(false)
    expect(value?.hasAttribute('data-tone')).toBe(false)
    expect(container.querySelector('.ui-card')).toBeNull()
  })

  it('StatTile：tone=success 只给数值徽章壳', async () => {
    const { container } = render(<StatTile label="就绪" value="3" hint="项" tone="success" />)
    const value = container.querySelector('.stat-tile__value')
    expect(value?.classList.contains('stat-tile__value--badge')).toBe(true)
    expect(value?.getAttribute('data-tone')).toBe('success')
    expect(container.querySelector('.stat-tile__tone-dot')).toBeTruthy()
    expect(container.querySelector('.ui-card')).toBeNull()
    const css = await readFile('src/ui/primitives.css', 'utf8')
    expect(css).toContain('tabular-nums')
  })
})
