import { describe, expect, it } from "vitest";
import {
  hasCardErrors,
  NO_CARD_ERRORS,
  validateCardFields,
} from "$lib/components/detail/cardValidation";
import { createEditForm } from "$lib/components/detail/editForm";

describe("validateCardFields", () => {
  it("accepts an empty form (card fields are optional)", () => {
    const errors = validateCardFields(createEditForm());
    expect(errors).toEqual(NO_CARD_ERRORS);
    expect(hasCardErrors(errors)).toBe(false);
  });

  it("accepts valid values", () => {
    const form = {
      ...createEditForm(),
      cardNumber: "4111111111111111",
      cardExpMonth: "12",
      cardExpYear: "2030",
      cardCvv: "123",
      cardPin: "4567",
    };
    expect(validateCardFields(form)).toEqual(NO_CARD_ERRORS);
  });

  it("judges by digits only so separators are ignored", () => {
    const form = { ...createEditForm(), cardNumber: "4111 1111 1111 1111" };
    expect(validateCardFields(form).cardNumber).toBe("");
  });

  it("rejects card numbers outside 13-19 digits", () => {
    const short = validateCardFields({ ...createEditForm(), cardNumber: "4111111" });
    expect(short.cardNumber).toBe("Card number too short");

    const long = validateCardFields({
      ...createEditForm(),
      cardNumber: "41111111111111111111",
    });
    expect(long.cardNumber).toBe("Card number too long");
  });

  it("rejects invalid expiry months", () => {
    expect(validateCardFields({ ...createEditForm(), cardExpMonth: "1" }).cardExpMonth).toBe(
      "Must be 2 digits (01-12)",
    );
    expect(validateCardFields({ ...createEditForm(), cardExpMonth: "13" }).cardExpMonth).toBe(
      "Must be 01-12",
    );
  });

  it("rejects short expiry years", () => {
    expect(validateCardFields({ ...createEditForm(), cardExpYear: "30" }).cardExpYear).toBe(
      "Year must be 4 digits",
    );
  });

  it("rejects CVVs that are not 3 or 4 digits", () => {
    expect(validateCardFields({ ...createEditForm(), cardCvv: "12" }).cardCvv).toBe(
      "CVV must be 3 or 4 digits",
    );
  });

  it("rejects PINs outside the ISO 9564 range", () => {
    expect(validateCardFields({ ...createEditForm(), cardPin: "123" }).cardPin).toBe(
      "PIN must be 4-12 digits",
    );
    expect(validateCardFields({ ...createEditForm(), cardPin: "1234567890123" }).cardPin).toBe(
      "PIN must be 4-12 digits",
    );
  });

  it("reports multiple errors at once", () => {
    const errors = validateCardFields({
      ...createEditForm(),
      cardNumber: "123",
      cardCvv: "12",
    });
    expect(errors.cardNumber).not.toBe("");
    expect(errors.cardCvv).not.toBe("");
    expect(hasCardErrors(errors)).toBe(true);
  });
});
