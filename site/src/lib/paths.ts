const base = import.meta.env.BASE_URL.replace(/\/$/, '');

export function href(path: string) {
  return `${base}${path}`;
}

export function docHref(id: string) {
  return href(`/docs/${id}/`);
}
