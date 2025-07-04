import Link from 'next/link';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Home, ArrowLeft, Leaf } from 'lucide-react';

export default function NotFound() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-900 dark:to-slate-800 flex items-center justify-center p-4">
      <Card className="w-full max-w-md text-center">
        <CardHeader className="space-y-4">
          <div className="mx-auto w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center">
            <Leaf className="h-8 w-8 text-primary" />
          </div>
          <div>
            <CardTitle className="text-2xl font-bold">Page Not Found</CardTitle>
            <CardDescription className="mt-2">
              The page you're looking for doesn't exist or has been moved.
            </CardDescription>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="text-6xl font-bold text-muted-foreground/20">404</div>
          <div className="space-y-3">
            <Button asChild className="w-full">
              <Link href="/">
                <Home className="h-4 w-4 mr-2" />
                Go Home
              </Link>
            </Button>
            <Button variant="outline" asChild className="w-full">
              <Link href="javascript:history.back()">
                <ArrowLeft className="h-4 w-4 mr-2" />
                Go Back
              </Link>
            </Button>
          </div>
          <div className="pt-4 border-t">
            <p className="text-sm text-muted-foreground">
              Need help? Visit our{' '}
              <Link href="/upload" className="text-primary hover:underline">
                upload page
              </Link>{' '}
              to get started with carbon tracking.
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}