<script lang="ts">
  // Rueda de color HSV sin librerías: el ángulo da el tono, la distancia al centro la saturación,
  // y el deslizador el brillo. Se dibuja con dos degradados CSS (conic = tonos, radial = blanco
  // en el centro) y una capa negra cuya opacidad es 1 − brillo. El campo HEX también vale para teclado.
  import { untrack } from 'svelte'
  import { t } from './i18n.svelte'

  let { value, onpick }: { value: string | null; onpick: (hex: string) => void } = $props()

  const start = untrack(() => (value && /^#[0-9a-f]{6}$/i.test(value) ? toHsv(value) : { h: 200, s: 0.7, v: 0.85 }))
  let h = $state(start.h)
  let s = $state(start.s)
  let v = $state(start.v)
  let hexInput = $state(toHex(start.h, start.s, start.v))
  const hex = $derived(toHex(h, s, v))
  let wheel: HTMLDivElement

  function toHex(h: number, s: number, v: number) {
    const f = (n: number) => {
      const k = (n + h / 60) % 6
      return Math.round(255 * (v - v * s * Math.max(0, Math.min(k, 4 - k, 1))))
    }
    return '#' + [f(5), f(3), f(1)].map((x) => x.toString(16).padStart(2, '0')).join('')
  }

  function toHsv(hex: string) {
    const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255)
    const max = Math.max(r, g, b)
    const d = max - Math.min(r, g, b)
    const h = d === 0 ? 0 : max === r ? ((g - b) / d + 6) % 6 : max === g ? (b - r) / d + 2 : (r - g) / d + 4
    return { h: h * 60, s: max === 0 ? 0 : d / max, v: max }
  }

  // puntero → tono (grados desde arriba, en el sentido del reloj, como el conic-gradient) y saturación
  function pickAt(e: PointerEvent) {
    const box = wheel.getBoundingClientRect()
    const dx = e.clientX - (box.left + box.width / 2)
    const dy = e.clientY - (box.top + box.height / 2)
    h = ((Math.atan2(dx, -dy) * 180) / Math.PI + 360) % 360
    s = Math.min(1, Math.hypot(dx, dy) / (box.width / 2))
    hexInput = hex
  }

  function onHexInput() {
    const typed = hexInput.trim().replace(/^#?/, '#')
    if (/^#[0-9a-f]{6}$/i.test(typed)) ({ h, s, v } = toHsv(typed.toLowerCase()))
  }

  // posición del marcador dentro de la rueda, en %
  const marker = $derived({
    x: 50 + 50 * s * Math.sin((h * Math.PI) / 180),
    y: 50 - 50 * s * Math.cos((h * Math.PI) / 180),
  })
</script>

<div class="picker">
  <div
    bind:this={wheel}
    class="wheel"
    role="presentation"
    onpointerdown={(e) => {
      wheel.setPointerCapture(e.pointerId)
      pickAt(e)
    }}
    onpointermove={(e) => e.buttons === 1 && pickAt(e)}
  >
    <div class="dark" style:opacity={1 - v}></div>
    <i class="marker" style:left="{marker.x}%" style:top="{marker.y}%" style:background={hex}></i>
  </div>
  <label class="row">
    <span>{t('edit.brightness')}</span>
    <input type="range" min="0" max="1" step="0.01" bind:value={v} oninput={() => (hexInput = hex)} />
  </label>
  <div class="row">
    <i class="preview" style:background={hex} aria-hidden="true"></i>
    <input class="hex" bind:value={hexInput} oninput={onHexInput} maxlength="7" spellcheck="false" aria-label={t('edit.hex')} />
    <button class="btn" onclick={() => onpick(hex)}>{t('edit.use_color')}</button>
  </div>
</div>

<style>
  .picker { display: grid; gap: 8px; justify-items: center; }
  .wheel {
    position: relative; width: 150px; aspect-ratio: 1; border-radius: 50%; cursor: crosshair; touch-action: none;
    background: radial-gradient(circle closest-side, #fff, transparent), conic-gradient(red, yellow, lime, aqua, blue, magenta, red);
  }
  .dark { position: absolute; inset: 0; border-radius: 50%; background: #000; pointer-events: none; }
  .marker { position: absolute; width: 14px; height: 14px; border-radius: 50%; border: 2px solid #fff; box-shadow: 0 0 0 1px #000; transform: translate(-50%, -50%); pointer-events: none; }
  .row { display: flex; gap: 6px; align-items: center; width: 100%; font-size: 13px; color: var(--muted); }
  .row input[type='range'] { flex: 1; width: 0; min-width: 0; } /* sin width: 0, el deslizador nativo exige ~130 px */
  .preview { width: 20px; height: 20px; border-radius: 50%; border: 1px solid var(--line); flex: none; }
  .hex { width: 9ch; min-width: 0; background: var(--pane); border: 1px solid var(--line); border-radius: 6px; padding: 3px 6px; font-variant-numeric: tabular-nums; }
  .row .btn { padding: 3px 8px; font-size: 13px; }
</style>
