import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid,
} from "recharts";


type Row = {
  id: number;
  kind: "expense" | "income";
  amount: number;
  category: string;
  note?: string | null;
  created_at: string;
};
type Summary = { month: string; income: number; expense: number; balance: number };
type CatRow = { category: string; total: number };

export default function App() {
  const [rows, setRows] = useState<Row[]>([]);
  const [kind, setKind] = useState<"expense" | "income">("expense");
  const [amount, setAmount] = useState("");
  const [category, setCategory] = useState("");
  const [note, setNote] = useState("");
  const [busyId, setBusyId] = useState<number | null>(null);
  const [ym, setYm] = useState(() => {
  const d = new Date();
  const mm = String(d.getMonth()+1).padStart(2, "0");
    return `${d.getFullYear()}-${mm}`;
  });
  const [summary, setSummary] = useState<Summary | null>(null);
  const [kindTab, setKindTab] = useState<"expense"|"income">("expense");
  const [cats, setCats] = useState<CatRow[]>([]);

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

  async function onDelete(id: number) {
    if (!confirm(`Delete entry #${id}?`)) return;
    try {
      setBusyId(id);
      const ok = (await invoke("delete", { id })) as boolean;
      if (!ok) alert("No entry deleted (already removed?)");
      await refresh();
    } finally {
      setBusyId(null);
    }
  }

  async function loadSummaryAndCats(y: string, k: "expense"|"income") {
    const s = (await invoke("get_month_summary", { ym: y })) as Summary;
    const c = (await invoke("get_category_totals", { ym: y, kind: k })) as CatRow[];
    setSummary(s);
    setCats(c);
  }

  useEffect(() => { // 初回
    loadSummaryAndCats(ym, kindTab);
    refresh(); // 既存の一覧ロード
  }, []);

  useEffect(() => { // 月 or タブが変わったら再ロード
    loadSummaryAndCats(ym, kindTab);
  }, [ym, kindTab]);

  useEffect(() => {
    refresh();
  }, []);

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

      <div style={{ display: "flex", gap: 12, alignItems: "center", marginBottom: 12 }}>
        <input
          type="month"
          value={ym}
          onChange={(e) => setYm(e.target.value)}
        />
        <div style={{ display: "flex", gap: 8 }}>
          <button
            onClick={() => setKindTab("expense")}
            style={{ padding: "6px 10px", background: kindTab==="expense" ? "#222" : "#eee", color: kindTab==="expense"?"#fff":"#000", border: "1px solid #ccc", borderRadius: 6 }}
          >Expense</button>
          <button
            onClick={() => setKindTab("income")}
            style={{ padding: "6px 10px", background: kindTab==="income" ? "#222" : "#eee", color: kindTab==="income"?"#fff":"#000", border: "1px solid #ccc", borderRadius: 6 }}
          >Income</button>
        </div>
      </div>

      {summary && (
        <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 12, marginBottom: 16 }}>
          <div style={{ padding: 12, border: "1px solid #ddd", borderRadius: 12 }}>
            <div style={{ color: "#666" }}>Income</div>
            <div style={{ fontSize: 24, fontWeight: 700 }}>{summary.income.toLocaleString()}</div>
          </div>
          <div style={{ padding: 12, border: "1px solid #ddd", borderRadius: 12 }}>
            <div style={{ color: "#666" }}>Expense</div>
            <div style={{ fontSize: 24, fontWeight: 700 }}>{summary.expense.toLocaleString()}</div>
          </div>
          <div style={{ padding: 12, border: "1px solid #ddd", borderRadius: 12 }}>
            <div style={{ color: "#666" }}>Balance</div>
            <div style={{ fontSize: 24, fontWeight: 700 }}>{summary.balance.toLocaleString()}</div>
          </div>
        </div>
      )}

      <div style={{ height: 280, border: "1px solid #eee", borderRadius: 12, padding: 8, marginBottom: 16 }}>
        <div style={{ padding: "0 8px 8px", fontWeight: 600 }}>
          Category totals ({kindTab}) – {ym}
        </div>
        <ResponsiveContainer width="100%" height="90%">
          <BarChart data={cats}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="category" />
            <YAxis />
            <Tooltip />
            <Bar dataKey="total" />
          </BarChart>
        </ResponsiveContainer>
      </div>


      <table style={{ borderCollapse: "collapse", width: "100%" }}>
        <thead>
          <tr>
            {["id", "kind", "amount", "category", "note", "created_at", "actions"].map((h) => (
              <th
                key={h}
                style={{
                  border: "1px solid #ddd",
                  padding: 8,
                  textAlign: "left",
                  background: "#f7f7f7",
                }}
              >
                {h}
              </th>
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
              <td style={{ border: "1px solid #ddd", padding: 8 }}>
                <button
                  onClick={() => onDelete(e.id)}
                  disabled={busyId === e.id}
                  title="Delete this entry"
                >
                  {busyId === e.id ? "Deleting..." : "Delete"}
                </button>
              </td>
            </tr>
          ))}
          {rows.length === 0 && (
            <tr>
              <td colSpan={7} style={{ padding: 16, textAlign: "center", color: "#666" }}>
                (no data)
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
