# This script terminates processes listening on specified TCP ports.

Write-Output '... Terminating old processes on ports 8081 and 3000 ...'

$ports = @(8081, 3000)

foreach ($port in $ports) {
    try {
        $conn = Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction Stop
        if ($conn) {
            $procId = $conn.OwningProcess
            Write-Output ('  - Terminating process on port ' + $port + ' (PID: ' + $procId + ') ...')
            Stop-Process -Id $procId -Force -ErrorAction SilentlyContinue
        }
    } catch [Microsoft.PowerShell.Commands.GetNetTCPConnectionCommand+CimException] {
        # This error occurs if no process is listening on the port, which is fine.
        Write-Output ('  - Port ' + $port + ' is not in use. No action needed.')
    } catch {
        Write-Error ("An unexpected error occurred while processing port {0}: {1}" -f $port, $_)
    }
}
