import { redirect } from 'next/navigation';

export default function HomePage() {
  // 重定向到配置管理页面
  redirect('/configs');
}

