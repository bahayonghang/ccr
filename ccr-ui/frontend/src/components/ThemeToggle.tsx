import { Moon, Sun } from 'lucide-react';
import { useEffect, useState } from 'react';

export default function ThemeToggle() {
  const [theme, setTheme] = useState<'light' | 'dark'>('light');

  useEffect(() => {
    // Load theme from localStorage
    const savedTheme = localStorage.getItem('ccr-theme') || 'light';
    setTheme(savedTheme as 'light' | 'dark');
    document.documentElement.setAttribute('data-theme', savedTheme);
  }, []);

  const toggleTheme = () => {
    const newTheme = theme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
    document.documentElement.setAttribute('data-theme', newTheme);
    localStorage.setItem('ccr-theme', newTheme);
  };

  return (
    <button
      onClick={toggleTheme}
      className="w-10 h-10 rounded-full bg-gray-100 dark:bg-gray-700 border-2 border-gray-300 dark:border-gray-600 hover:border-primary dark:hover:border-primary transition-all hover:rotate-180 hover:scale-110 flex items-center justify-center"
      title="Toggle theme"
    >
      {theme === 'dark' ? (
        <Moon className="w-5 h-5 text-gray-600 dark:text-gray-300" />
      ) : (
        <Sun className="w-5 h-5 text-gray-600 dark:text-gray-300" />
      )}
    </button>
  );
}

