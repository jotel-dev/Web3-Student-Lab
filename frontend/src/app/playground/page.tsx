export default function PlaygroundPage() {
  return (
    <div className="min-h-[calc(100vh-80px)] bg-black text-white p-12 relative overflow-hidden">
      <div className="absolute top-0 right-0 w-[500px] h-[500px] bg-red-600/10 rounded-full blur-[100px] pointer-events-none"></div>
      <div className="max-w-4xl mx-auto relative z-10 border-l-4 border-red-600 pl-8 py-4">
        <h1 className="text-4xl md:text-5xl font-black uppercase tracking-tight mb-4">
          Contract <span className="text-red-500">Playground</span>
        </h1>
        <p className="text-xl text-gray-400 font-light mb-8 font-mono">
          System Status: Sandbox allocation pending...
        </p>
        <div className="bg-zinc-950 border border-white/10 p-8 rounded-xl shadow-[0_0_30px_rgba(220,38,38,0.05)]">
          <p className="text-gray-500 tracking-wide">Write and test smart contracts directly in the browser.</p>
        </div>
      </div>
    </div>
  );
}
