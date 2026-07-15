import { SUPPORTED_LANGUAGES, type LanguageCode } from "../translation/language";

interface LanguagePairSelectorProps {
  sourceLanguage: LanguageCode;
  targetLanguage: LanguageCode;
  onSwap: () => void;
  disabled?: boolean;
}

export function LanguagePairSelector({
  sourceLanguage,
  targetLanguage,
  onSwap,
  disabled,
}: LanguagePairSelectorProps) {
  return (
    <div className="field">
      <span className="field-label">Translation</span>
      <div className="language-pair">
        <select className="field-control" value={sourceLanguage} disabled>
          {SUPPORTED_LANGUAGES.map((l) => (
            <option key={l.code} value={l.code}>
              {l.label}
            </option>
          ))}
        </select>
        <button
          type="button"
          className="swap-button"
          onClick={onSwap}
          disabled={disabled}
          aria-label="Swap languages"
          title="Swap languages"
        >
          &#8595;&#8593;
        </button>
        <select className="field-control" value={targetLanguage} disabled>
          {SUPPORTED_LANGUAGES.map((l) => (
            <option key={l.code} value={l.code}>
              {l.label}
            </option>
          ))}
        </select>
      </div>
    </div>
  );
}
