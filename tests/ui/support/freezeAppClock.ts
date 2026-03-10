import type { Page } from "@playwright/test";

export async function freezeAppClock(page: Page) {
  const fixedNow = new Date("2026-02-03T12:00:00.000Z").valueOf();
  await page.addInitScript((timestamp) => {
    const RealDate = Date;

    class FixedDate extends RealDate {
      constructor(...args: ConstructorParameters<typeof Date>) {
        if (args.length === 0) {
          super(timestamp);
          return;
        }
        super(...args);
      }

      static now() {
        return timestamp;
      }
    }

    FixedDate.parse = RealDate.parse;
    FixedDate.UTC = RealDate.UTC;
    Object.setPrototypeOf(FixedDate, RealDate);
    // @ts-expect-error test-only Date shim
    window.Date = FixedDate;
  }, fixedNow);
}
