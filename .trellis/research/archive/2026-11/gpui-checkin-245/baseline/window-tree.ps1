param([long]$WindowHandle, [switch]$Open)
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$root = [System.Windows.Automation.AutomationElement]::FromHandle([IntPtr]::new($WindowHandle))
$tree = [System.Windows.Automation.TreeScope]::Descendants
$all = $root.FindAll($tree,[System.Windows.Automation.Condition]::TrueCondition)
$nav = [string]([char]0x7B7E) + [string]([char]0x5230) + [string]([char]0x7BA1) + [string]([char]0x7406)
$rows = foreach ($el in $all) {
    $c = $el.Current
    if ($Open -and $c.Name -eq $nav -and $c.ControlType.ProgrammaticName -eq 'ControlType.Button') {
        $invoke = $null
        if ($el.TryGetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern, [ref]$invoke)) {
            $invoke.Invoke()
            'invoked'
            return
        }
    }
    if ($c.Name) {
        [PSCustomObject]@{name=$c.Name; role=$c.ControlType.ProgrammaticName; left=$c.BoundingRectangle.Left; top=$c.BoundingRectangle.Top; width=$c.BoundingRectangle.Width; height=$c.BoundingRectangle.Height }
    }
}
ConvertTo-Json -InputObject @($rows) -Depth 2 -Compress
