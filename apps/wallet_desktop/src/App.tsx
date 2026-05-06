import { AppRouter } from "./app/router/AppRouter";
import { WalletProvider } from "./app/providers/WalletProvider";

function App() {
  return (
    <WalletProvider>
      <AppRouter />
    </WalletProvider>
  );
}

export default App;
