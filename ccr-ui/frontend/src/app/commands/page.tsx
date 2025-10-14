'use client';

import { redirect } from 'next/navigation';

export default function CommandsPage() {
  // 重定向到 CCR 命令页面
  redirect('/commands/ccr');
}
