function App() {
  return (
    <main className="flex justify-center w-full min-h-screen items-center bg-neutral-900">
      <h1 className="text-3xl text-white font-bold">Hello world</h1>
      <h1 className="text-3xl text-white font-bold">
        Build time url: {import.meta.env.VITE_DOMAIN}
      </h1>
    </main>
  );
}

export default App;
