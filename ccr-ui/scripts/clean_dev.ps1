# This script terminates processes listening on specified TCP ports.

Write-Output '... Terminating old processes on ports 8081, 5173, 5174, and 3000 ...'

$ports = @(8081, 5173, 5174, 3000)

foreach ($port in $ports) {
    try {
        $conn = Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction Stop 2>$null
        if ($conn) {
            $procId = $conn.OwningProcess
            Write-Output ('  - Terminating process on port ' + $port + ' (PID: ' + $procId + ') ...')
            Stop-Process -Id $procId -Force -ErrorAction SilentlyContinue
        }
    } catch {
        # Port is not in use or other error - this is fine
        Write-Output ('  - Port ' + $port + ' is not in use. No action needed.')
    }
}
