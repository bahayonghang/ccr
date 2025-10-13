import { Routes, Route, Navigate } from 'react-router-dom';
import ConfigManagement from './pages/ConfigManagement';
import CommandExecutor from './pages/CommandExecutor';

function App() {
  return (
    <div className="min-h-screen" style={{ background: 'var(--bg-primary)' }}>
      <Routes>
        <Route path="/" element={<Navigate to="/configs" replace />} />
        <Route path="/configs" element={<ConfigManagement />} />
        <Route path="/commands" element={<CommandExecutor />} />
      </Routes>
    </div>
  );
}

export default App;

