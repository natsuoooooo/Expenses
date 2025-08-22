import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Row = {
  id: number;
  kind: "expense" | "income";
  amount: number;
  category: string;
  note?: string | null;
  created_at: string;
};

export default function App() {
  const [rows, setRows] = useState<Row[]>([]);
  const [kind, setKind] = useState<"expense" | "income">("expense");
  const [amount, setAmount] = useState<string>("");
  const [category, setCategory] = useState<string>("");
  const [note, setNote] = useState<string>("");

  async function refresh() {
    const res = (await invoke("list")) as Row[];
    setRows(res);
  }

  async function onAdd() {
    const amt = Number(amount);
    if (!amt || amt <= 0 || !category.trim()) {
      alert("amount>0 & category required");
      return;
    }
    await invoke("add", {
      kind,
      amount: amt,
      category,
      note: note.trim() === "" ? null : note,
    });
    setAmount("");
    setCategory("");
    setNote("");
    await refresh();
  }

  useEffect(() => { refresh(); }, []);

  return (
    <div style={{ fontFamily: "system-ui, sans-serif", padding: 24 }}>
      <h1>Ledger (Desktop)</h1>

      <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginBottom: 12 }}>
        <select value={kind} onChange={(e) => setKind(e.target.value as any)}>
          <option value="expense">expense</option>
          <option value="income">income</option>
        </select>
        <input
          type="number"
          placeholder="amount"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
        />
        <input
          type="text"
          placeholder="category"
          value={category}
          onChange={(e) => setCategory(e.target.value)}
        />
        <input
          type="text"
          placeholder="note (optional)"
          value={note}
          onChange={(e) => setNote(e.target.value)}
        />
        <button onClick={onAdd}>Add</button>
        <button onClick={refresh}>Reload</button>
      </div>

      <table style={{ borderCollapse: "collapse", width: "100%" }}>
        <thead>
          <tr>
            {["id", "kind", "amount", "category", "note", "created_at"].map((h) => (
              <th key={h} style={{ border: "1px solid #ddd", padding: 8, textAlign: "left", background: "#f7f7f7" }}>{h}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((e) => (
            <tr key={e.id}>
              <td style={{ border: "1px solid #ddd", padding: 8 }}>{e.id}</td>
              <td style={{ border: "1px solid #ddd", padding: 8 }}>{e.kind}</td>
              <td style={{ border: "1px solid #ddd", padding: 8 }}>{e.amount}</td>
              <td style={{ border: "1px solid #ddd", padding: 8 }}>{e.category}</td>
              <td style={{ border: "1px solid #ddd", padding: 8 }}>{e.note ?? ""}</td>
              <td style={{ border: "1px solid #ddd", padding: 8 }}>{e.created_at}</td>
            </tr>
          ))}
          {rows.length === 0 && (
            <tr><td colSpan={6} style={{ padding: 16, textAlign: "center", color: "#666" }}>(no data)</td></tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
