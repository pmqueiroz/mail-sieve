export type Route = 'overview' | 'senders' | 'messages' | 'rules' | 'activity' | 'accounts' | 'plugins';

export const routes: { route: Route; label: string }[] = [
  { route: 'overview', label: 'Overview' },
  { route: 'senders', label: 'Senders' },
  { route: 'messages', label: 'Messages' },
  { route: 'rules', label: 'Rules' },
  { route: 'activity', label: 'Activity' },
  { route: 'accounts', label: 'Accounts' },
  { route: 'plugins', label: 'Plugins' },
];

function parse(): { route: Route; params: URLSearchParams } {
  const hash = window.location.hash.replace(/^#\/?/, '');
  const [path = '', search = ''] = hash.split('?');
  const known = routes.find((entry) => entry.route === path);
  return { route: known?.route ?? 'overview', params: new URLSearchParams(search) };
}

class Router {
  route = $state<Route>('overview');
  params = $state(new URLSearchParams());

  constructor() {
    this.sync();
    window.addEventListener('hashchange', () => this.sync());
  }

  private sync() {
    const { route, params } = parse();
    this.route = route;
    this.params = params;
  }

  go(route: Route, params: Record<string, string> = {}) {
    const search = new URLSearchParams(params).toString();
    window.location.hash = `#/${route}${search ? `?${search}` : ''}`;
  }
}

export const router = new Router();
