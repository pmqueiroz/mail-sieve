export type Route = 'senders' | 'messages' | 'rules' | 'activity' | 'accounts';

const routes: Route[] = ['senders', 'messages', 'rules', 'activity', 'accounts'];

function parse(): { route: Route; params: URLSearchParams } {
  const hash = window.location.hash.replace(/^#\/?/, '');
  const [path = '', search = ''] = hash.split('?');
  const route = routes.includes(path as Route) ? (path as Route) : 'senders';
  return { route, params: new URLSearchParams(search) };
}

class Router {
  route = $state<Route>('senders');
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
