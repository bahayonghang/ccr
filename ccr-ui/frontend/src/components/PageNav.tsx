import { NavLink } from 'react-router-dom';
import { Settings, Terminal } from 'lucide-react';

const navigation = [
  { name: '配置管理', to: '/configs', icon: Settings },
  { name: '命令执行', to: '/commands', icon: Terminal },
];

export default function PageNav() {
  return (
    <div
      className="flex gap-2 mb-5 p-1 rounded-lg"
      style={{ background: 'var(--bg-tertiary)' }}
    >
      {navigation.map((item) => (
        <NavLink
          key={item.to}
          to={item.to}
          className={({ isActive }) =>
            `flex-1 flex items-center justify-center space-x-2 px-4 py-2.5 rounded-md text-sm font-semibold transition-all ${
              isActive ? 'text-white' : ''
            }`
          }
          style={({ isActive }) => ({
            background: isActive
              ? 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))'
              : 'transparent',
            color: isActive ? 'white' : 'var(--text-secondary)',
            boxShadow: isActive ? '0 0 15px var(--glow-primary)' : undefined,
          })}
        >
          <item.icon className="w-4 h-4" />
          <span>{item.name}</span>
        </NavLink>
      ))}
    </div>
  );
}

