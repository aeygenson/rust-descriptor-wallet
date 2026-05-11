export const routes = {
  overview: "/",
  receive: "/receive",
  addressBook: "/address-book",
  utxos: "/utxos",
  send: "/send",
  transactions: "/transactions",
} as const;

export type RouteId = keyof typeof routes;

export type NavigationItem = {
  id: RouteId;
  label: string;
  path: (typeof routes)[RouteId];
};

export const navigationItems: NavigationItem[] = [
  { id: "overview", label: "Overview", path: routes.overview },
  { id: "receive", label: "Receive", path: routes.receive },
  { id: "addressBook", label: "Address Book", path: routes.addressBook },
  { id: "utxos", label: "UTXOs", path: routes.utxos },
  { id: "send", label: "Send", path: routes.send },
  { id: "transactions", label: "Transactions", path: routes.transactions },
];

export function pathFor(route: RouteId): string {
  return routes[route];
}