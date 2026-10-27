import { Outlet } from 'react-router'
import { useAppT } from '@/i18n'
import { getWindowChromeTopInset, shouldUseCustomTitlebar } from '@/utils/windowChrome'
import { GlobalConfirmDialog } from './GlobalConfirmDialog'
import { StageBackground } from './StageBackground'
import { Titlebar } from './Titlebar'
import { ToastContainer } from './ToastContainer'
import { useRouteHandle } from './routeHandle'
import { useShellRuntime } from './useShellRuntime'

export function App() {
  useAppT()
  useShellRuntime()
  const handle = useRouteHandle()
  const showCustomTitlebar = shouldUseCustomTitlebar()
  const inset = getWindowChromeTopInset()

  return (
    <>
      {handle.hideGlobalBackground ? null : <StageBackground />}
      {showCustomTitlebar ? <Titlebar /> : null}
      <div
        className="app-field flex h-screen w-screen flex-col overflow-hidden"
        style={inset > 0 ? { paddingTop: `${inset}px` } : undefined}
      >
        <Outlet />
      </div>
      <ToastContainer />
      <GlobalConfirmDialog />
    </>
  )
}
