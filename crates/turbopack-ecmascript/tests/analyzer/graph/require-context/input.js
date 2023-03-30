function importAll(r) {
  return r.keys().map(r);
}

const items = importAll(require.context('./test', false, /\.test\.js$/));
