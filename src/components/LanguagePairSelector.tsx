import { SUPPORTED_LANGUAGES, type LanguageCode } from "../translation/language";
import { SwapIcon } from "./icons";

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
    <section>
      <p className="section-label">Translation</p>
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
          <SwapIcon size={14} />
        </button>
        <select className="field-control" value={targetLanguage} disabled>
          {SUPPORTED_LANGUAGES.map((l) => (
            <option key={l.code} value={l.code}>
              {l.label}
            </option>
          ))}
        </select>
      </div>
    </section>
  );
}
