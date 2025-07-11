'use client';

import { useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { authService } from '@/lib/auth';

export default function AuthCallback() {
  const router = useRouter();

  useEffect(() => {
    const handleCallback = async () => {
      try {
        const user = await authService.handleOAuthCallback();
        if (user) {
          // Redirect to dashboard on successful authentication
          router.push('/');
        } else {
          // Redirect to home with error if authentication failed
          router.push('/?error=auth_failed');
        }
      } catch (error) {
        console.error('Auth callback error:', error);
        router.push('/?error=auth_failed');
      }
    };

    handleCallback();
  }, [router]);

  return (
    <div className="min-h-screen flex items-center justify-center">
      <div className="text-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
        <p className="text-muted-foreground">Completing authentication...</p>
      </div>
    </div>
  );
}