import { useSignal } from "preact/signals"

interface Props {
  did: string
  targetChannelId: string
}

export default function SettingsForm(props: Props) {
  const did = useSignal(props.did)
  const targetChannelId = useSignal(props.targetChannelId)
  const message = useSignal("")

  const handleSubmit = async (e: Event) => {
    e.preventDefault()
    message.value = ""

    const res = await fetch("/api/settings", {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        did: did.value,
        targetChannelId: targetChannelId.value,
      }),
    })

    message.value = res.ok ? "Saved" : "Failed to save"
  }

  return (
    <form onSubmit={handleSubmit}>
      <div>
        <label>
          DID:
          <input
            type="text"
            value={did.value}
            onInput={(e) => (did.value = e.currentTarget.value)}
          />
        </label>
      </div>
      <div>
        <label>
          Target Channel ID:
          <input
            type="text"
            value={targetChannelId.value}
            onInput={(e) => (targetChannelId.value = e.currentTarget.value)}
          />
        </label>
      </div>
      <button type="submit">Save</button>
      {message.value && <p>{message.value}</p>}
    </form>
  )
}
