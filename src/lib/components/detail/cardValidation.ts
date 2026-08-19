import type { EditFormState } from "$lib/components/detail/editForm";

/** Per-field card validation errors; empty string means valid. */
export interface CardErrors {
  cardNumber: string;
  cardExpMonth: string;
  cardExpYear: string;
  cardCvv: string;
  cardPin: string;
}

export const NO_CARD_ERRORS: CardErrors = {
  cardNumber: "",
  cardExpMonth: "",
  cardExpYear: "",
  cardCvv: "",
  cardPin: "",
};

/**
 * Validate the card fields of an edit form. Judge by digits only: pasted or
 * programmatically set values can carry spaces/separators (and letters) that
 * the oninput sanitizers miss.
 */
export function validateCardFields(form: EditFormState): CardErrors {
  const digits = (value: string) => value.replace(/\D/g, "");

  // Card number: digits only, 13-19 chars (standard card lengths)
  const cardNumber = digits(form.cardNumber);
  const cardNumberError =
    cardNumber.length > 0 && (cardNumber.length < 13 || cardNumber.length > 19)
      ? cardNumber.length < 13
        ? "Card number too short"
        : "Card number too long"
      : "";

  // Exp month: 2 digits, 01-12
  const expMonth = digits(form.cardExpMonth);
  let cardExpMonthError = "";
  if (expMonth && expMonth.length !== 2) {
    cardExpMonthError = "Must be 2 digits (01-12)";
  } else if (expMonth) {
    const m = Number.parseInt(expMonth, 10);
    if (m < 1 || m > 12) cardExpMonthError = "Must be 01-12";
  }

  // Exp year: 4 digits
  const cardExpYearError =
    form.cardExpYear && digits(form.cardExpYear).length !== 4 ? "Year must be 4 digits" : "";

  // CVV: 3 or 4 digits
  const cvv = digits(form.cardCvv);
  const cardCvvError =
    cvv && cvv.length !== 3 && cvv.length !== 4 ? "CVV must be 3 or 4 digits" : "";

  // PIN: 4-12 digits (ISO 9564 range)
  const pin = digits(form.cardPin);
  const cardPinError = pin && (pin.length < 4 || pin.length > 12) ? "PIN must be 4-12 digits" : "";

  return {
    cardNumber: cardNumberError,
    cardExpMonth: cardExpMonthError,
    cardExpYear: cardExpYearError,
    cardCvv: cardCvvError,
    cardPin: cardPinError,
  };
}

export function hasCardErrors(errors: CardErrors): boolean {
  return Object.values(errors).some((error) => error !== "");
}
