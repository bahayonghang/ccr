import type { Metadata, Viewport } from 'next';
import { Inter } from 'next/font/google';
import './globals.css';
import { ThemeProvider } from '@/components/providers/ThemeProvider';

const inter = Inter({
  subsets: ['latin'],
  display: 'swap',
  variable: '--font-inter',
});

export const metadata: Metadata = {
  title: 'CCR UI - Claude Code Configuration Switcher',
  description: 'Web console for managing Claude Code configurations',
  keywords: ['Claude Code', 'Configuration', 'API', 'Management'],
  authors: [{ name: 'CCR Team' }],
};

export const viewport: Viewport = {
  width: 'device-width',
  initialScale: 1,
  maximumScale: 5,
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="zh-CN" suppressHydrationWarning>
      <head>
        <meta charSet="UTF-8" />
        <link rel="icon" type="image/svg+xml" href="/vite.svg" />
      </head>
      <body className={`${inter.variable} font-sans antialiased`}>
        <ThemeProvider>
          {/* 动态背景效果 */}
          <div className="bg-effect" aria-hidden="true" />
          {children}
        </ThemeProvider>
      </body>
    </html>
  );
}

