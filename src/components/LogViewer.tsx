import { useEffect, useRef } from "react";

interface Props {
  logs: string[];
}

function LogViewer({ logs }: Props) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs]);

  return (
    <div
      ref={scrollRef}
      className="bg-gray-950 border border-gray-800 rounded-lg p-4 h-48 overflow-y-auto font-mono text-xs"
    >
      {logs.length === 0 ? (
        <p className="text-gray-600">Waiting for events...</p>
      ) : (
        logs.map((log, i) => (
          <div key={i} className="text-gray-400 leading-relaxed">
            <span className="text-gray-600">[{String(i + 1).padStart(2, "0")}]</span> {log}
          </div>
        ))
      )}
    </div>
  );
}

export default LogViewer;
