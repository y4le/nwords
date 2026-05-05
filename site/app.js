import init, {
  decode_id_json,
  decode_text_json,
  demo_info_json,
  encode_id_json,
  encode_text_json,
  plan_json,
  presets_json,
} from "./pkg/nwords_web.js";

const state = {
  presets: [],
  ready: false,
};

const el = {
  preset: document.querySelector("#preset-select"),
  idInput: document.querySelector("#id-input"),
  phraseInput: document.querySelector("#phrase-input"),
  idOutput: document.querySelector("#id-output"),
  statsList: document.querySelector("#stats-list"),
  rangeInput: document.querySelector("#range-input"),
  wordsInput: document.querySelector("#words-input"),
  planOutput: document.querySelector("#plan-output"),
  textInput: document.querySelector("#text-input"),
  textPhraseInput: document.querySelector("#text-phrase-input"),
  textOutput: document.querySelector("#text-output"),
  compareGrid: document.querySelector("#compare-grid"),
};

await init();
state.ready = true;

const info = call(demo_info_json);
if (info.ok) {
  document.querySelector("#positional-caveat").textContent = info.positional_caveat;
  document.querySelector("#spread-caveat").textContent = info.spread_caveat;
  document.querySelector("#text-caveat").textContent = info.text_caveat;
}

const presets = call(presets_json);
if (presets.ok) {
  state.presets = presets.presets;
  for (const preset of state.presets) {
    const option = document.createElement("option");
    option.value = preset.name;
    option.textContent = preset.name;
    el.preset.append(option);
  }
  el.preset.value = "dec6";
}

bindEvents();
encodeId();
planShape();
encodeText();
compareNearby();

function bindEvents() {
  document.querySelector("#encode-button").addEventListener("click", encodeId);
  document.querySelector("#decode-button").addEventListener("click", decodeId);
  document.querySelector("#compare-button").addEventListener("click", compareNearby);
  document.querySelector("#plan-button").addEventListener("click", planShape);
  document.querySelector("#text-encode-button").addEventListener("click", encodeText);
  document.querySelector("#text-decode-button").addEventListener("click", decodeText);
  el.preset.addEventListener("change", () => {
    encodeId();
    compareNearby();
  });
}

function encodeId() {
  const result = call(encode_id_json, el.idInput.value, el.preset.value);
  if (result.ok) {
    el.phraseInput.value = result.phrase;
    renderResult(el.idOutput, [
      ["phrase", result.phrase],
      ["id", result.id],
      ["preset", result.preset],
    ]);
    renderStats(result);
  } else {
    renderError(el.idOutput, result.error);
  }
}

function decodeId() {
  const result = call(decode_id_json, el.phraseInput.value, el.preset.value);
  if (result.ok) {
    el.idInput.value = result.id;
    renderResult(el.idOutput, [
      ["id", result.id],
      ["phrase", result.phrase],
      ["preset", result.preset],
    ]);
    renderStats(result);
  } else {
    renderError(el.idOutput, result.error);
  }
}

function planShape() {
  const result = call(plan_json, el.rangeInput.value, el.wordsInput.value);
  if (result.ok) {
    renderResult(el.planOutput, [
      ["words", String(result.words)],
      ["range", result.range],
      ["capacity", result.capacity],
      ["slack", result.slack ?? "n/a"],
      ["acceptance", result.acceptance_ratio ?? "n/a"],
    ]);
  } else {
    renderError(el.planOutput, result.error);
  }
}

function encodeText() {
  const result = call(encode_text_json, el.textInput.value);
  if (result.ok) {
    el.textPhraseInput.value = result.phrase;
    renderResult(el.textOutput, [
      ["phrase", result.phrase],
      ["bytes", String(result.byte_length)],
      ["scheme", result.scheme],
    ]);
  } else {
    renderError(el.textOutput, result.error);
  }
}

function decodeText() {
  const result = call(decode_text_json, el.textPhraseInput.value);
  if (result.ok) {
    el.textInput.value = result.text;
    renderResult(el.textOutput, [
      ["text", result.text],
      ["bytes", String(result.byte_length)],
      ["scheme", result.scheme],
    ]);
  } else {
    renderError(el.textOutput, result.error);
  }
}

function compareNearby() {
  const start = toBigInt(el.idInput.value, 0n);
  const selected = el.preset.value;
  const base = selected.endsWith("-spread") ? selected.slice(0, -7) : selected;
  const spread = `${base}-spread`;
  const presets = state.presets.some((preset) => preset.name === spread)
    ? [base, spread]
    : [selected];

  el.compareGrid.replaceChildren();
  for (const preset of presets) {
    const column = document.createElement("div");
    column.className = "compare-column";
    const heading = document.createElement("h3");
    heading.textContent = preset;
    column.append(heading);
    for (let offset = 0n; offset < 4n; offset += 1n) {
      const id = start + offset;
      const result = call(encode_id_json, id.toString(), preset);
      const row = document.createElement("div");
      row.className = "compare-row";
      const idCell = document.createElement("span");
      idCell.textContent = id.toString();
      const phraseCell = document.createElement("span");
      phraseCell.textContent = result.ok ? result.phrase : result.error;
      row.append(idCell, phraseCell);
      column.append(row);
    }
    el.compareGrid.append(column);
  }
}

function renderStats(result) {
  const rows = [
    ["preset", result.preset],
    ["dictionary", result.dictionary],
    ["permutation", result.permutation],
    ["words", String(result.words)],
    ["range", result.range],
    ["capacity", result.capacity],
    ["slack", result.slack ?? "n/a"],
    ["acceptance", result.acceptance_ratio ?? "n/a"],
  ];
  el.statsList.replaceChildren();
  for (const [name, value] of rows) {
    const term = document.createElement("dt");
    term.textContent = name;
    const detail = document.createElement("dd");
    detail.textContent = value;
    el.statsList.append(term, detail);
  }
}

function renderResult(target, rows) {
  target.classList.remove("error");
  target.replaceChildren();
  for (const [name, value] of rows) {
    const row = document.createElement("div");
    const label = document.createElement("span");
    label.textContent = name;
    const text = document.createElement("strong");
    text.textContent = value;
    row.append(label, text);
    target.append(row);
  }
}

function renderError(target, message) {
  target.classList.add("error");
  target.textContent = message;
}

function call(fn, ...args) {
  try {
    return JSON.parse(fn(...args));
  } catch (error) {
    return { ok: false, error: error instanceof Error ? error.message : String(error) };
  }
}

function toBigInt(value, fallback) {
  try {
    const trimmed = value.trim();
    return trimmed === "" ? fallback : BigInt(trimmed);
  } catch {
    return fallback;
  }
}
