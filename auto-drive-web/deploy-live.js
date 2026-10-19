import { execSync } from 'child_process';

try {
  console.log('Building project with Vite...');
  execSync('pnpm run translations', { stdio: 'inherit' });
  execSync('pnpm run build', { stdio: 'inherit' });

  console.log('Finding target pod...');
  const podName = execSync(
    'kubectl get pod -l app=autodrive-web -o jsonpath="{.items[0].metadata.name}"',
    { encoding: 'utf8' }
  ).trim();

  if (!podName) {
    throw new Error('No pod found matching label app=autodrive-web');
  }

  console.log(`Targeting pod: ${podName}`);

  console.log('Clearing old assets in /usr/share/nginx/html/...');
  execSync(`kubectl exec ${podName} -- sh -c "rm -rf /usr/share/nginx/html/*"`, { stdio: 'inherit' });

  console.log('Copying dist/ files to pod...');
  execSync(`kubectl cp dist/. ${podName}:/usr/share/nginx/html/`, { stdio: 'inherit' });

  console.log('Successfully updated live pod!');
} catch (error) {
  console.error('Live deployment failed:', error.message);
  process.exit(1);
}
