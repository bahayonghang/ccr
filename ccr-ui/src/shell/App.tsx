import { Outlet } from 'react-router'
import { getWindowChromeTopInset, shouldUseCustomTitlebar } from '@/utils/windowChrome'
import { GlobalConfirmDialog } from './GlobalConfirmDialog'
import { StageBackground } from './StageBackground'
import { Titlebar } from './Titlebar'
import { ToastContainer } from './ToastContainer'
import { useRouteHandle } from './routeHandle'
import { useShellRuntime } from './useShellRuntime'

export function App() {
  useShellRuntime()
  const handle = useRouteHandle()
  const showCustomTitlebar = shouldUseCustomTitlebar()
  const inset = getWindowChromeTopInset()

  return (
    <>
      {handle.hideGlobalBackground ? null : <StageBackground />}
      {showCustomTitlebar ? <Titlebar /> : null}
      <div
        className="flex h-screen w-screen flex-col overflow-hidden bg-bg-base"
        style={inset > 0 ? { paddingTop: `${inset}px` } : undefined}
      >
        <Outlet />
      </div>
      <ToastContainer />
      <GlobalConfirmDialog />
    </>
  )
}
