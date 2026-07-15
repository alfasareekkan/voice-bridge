export type LanguageCode = "en" | "ml";

export interface LanguageOption {
  code: LanguageCode;
  label: string;
}

export const SUPPORTED_LANGUAGES: LanguageOption[] = [
  { code: "en", label: "English" },
  { code: "ml", label: "Malayalam" },
];

export function languageLabel(code: LanguageCode): string {
  return SUPPORTED_LANGUAGES.find((l) => l.code === code)?.label ?? code;
}
