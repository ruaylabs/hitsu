import type { EntryEditPayload, EntryPatch } from "$lib/bridge/entries";
import type { Entry } from "$lib/bridge/types";

export function createEditForm(entry?: Entry, secrets?: EntryEditPayload | null) {
  return {
    title: entry?.title ?? "",
    username: entry?.username ?? "",
    password: secrets?.password ?? "",
    url: entry?.url ?? "",
    totp: secrets?.totp ?? "",
    notes: entry?.notes ?? "",
    expiresAt: entry?.expiresAt ?? "",
    tags: [...(entry?.tags ?? [])],
    customFields: (secrets?.customFields ?? entry?.customFields ?? []).map((field) => ({
      ...field,
    })),
    firstName: entry?.identity?.firstName ?? "",
    lastName: entry?.identity?.lastName ?? "",
    email: entry?.identity?.email ?? "",
    phone: entry?.identity?.phone ?? "",
    address: entry?.identity?.address ?? "",
    dob: entry?.identity?.dob ?? "",
    cardHolder: entry?.card?.holder ?? "",
    cardNumber: secrets?.cardNumber ?? "",
    cardType: entry?.card?.type ?? "",
    cardExpMonth: entry?.card?.expMonth?.toString() ?? "",
    cardExpYear: entry?.card?.expYear?.toString() ?? "",
    cardCvv: secrets?.cardCvv ?? "",
    cardPin: secrets?.cardPin ?? "",
    licenseVersion: entry?.softwareLicense?.version ?? "",
    licenseKey: secrets?.licenseKey ?? "",
    licenseLicensedTo: entry?.softwareLicense?.licensedTo ?? "",
    licenseRegisteredEmail: entry?.softwareLicense?.registeredEmail ?? "",
    licenseCompany: entry?.softwareLicense?.company ?? "",
    licenseDownloadPage: entry?.softwareLicense?.downloadPage ?? "",
    licensePublisher: entry?.softwareLicense?.publisher ?? "",
    licenseWebsite: entry?.softwareLicense?.website ?? "",
    licenseRetailPrice: entry?.softwareLicense?.retailPrice ?? "",
    licenseSupportEmail: entry?.softwareLicense?.supportEmail ?? "",
    licensePurchaseDate: entry?.softwareLicense?.purchaseDate ?? "",
    licenseOrderNumber: entry?.softwareLicense?.orderNumber ?? "",
    licenseOrderTotal: entry?.softwareLicense?.orderTotal ?? "",
    passportType: entry?.passport?.type ?? "",
    passportIssuingCountry: entry?.passport?.issuingCountry ?? "",
    passportNumber: secrets?.passportNumber ?? "",
    passportFullName: entry?.passport?.fullName ?? "",
    passportSex: entry?.passport?.sex ?? "",
    passportNationality: entry?.passport?.nationality ?? "",
    passportIssuingAuthority: entry?.passport?.issuingAuthority ?? "",
    passportBirthDate: entry?.passport?.birthDate ?? "",
    passportBirthPlace: entry?.passport?.birthPlace ?? "",
    passportIssueDate: entry?.passport?.issueDate ?? "",
    passportExpiryDate: entry?.passport?.expiryDate ?? "",
    pgpPublicKey: entry?.pgpKey?.publicKey ?? "",
    pgpPrivateKey: secrets?.pgpPrivateKey ?? "",
    pgpFingerprint: entry?.pgpKey?.fingerprint ?? "",
    pgpKeyId: entry?.pgpKey?.keyId ?? "",
    pgpUserIds: entry?.pgpKey?.userIds ?? "",
    pgpAlgorithm: entry?.pgpKey?.algorithm ?? "",
    pgpExpiresAt: entry?.pgpKey?.expiresAt ?? "",
  } satisfies EntryPatch;
}

export type EditFormState = ReturnType<typeof createEditForm>;

export function cloneEditForm(form: EditFormState): EditFormState {
  return {
    ...form,
    tags: [...form.tags],
    customFields: form.customFields.map((field) => ({ ...field })),
  };
}
