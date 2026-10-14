// Formats text from a mod's own catalogs in the launcher (setting labels, hotkey names).

import { FluentBundle, FluentResource } from '@fluent/bundle';
import { negotiateLanguages } from '@fluent/langneg';

export type ModTranslator = (id: string) => string;

export async function loadModTranslator(
  modBase: string,
  modId: string,
  locale: string,
): Promise<ModTranslator> {
  const root = `${modBase}/${modId}`;
  try {
    const listing = (await (await fetch(`${root}/.overlay/locales.json`)).json()) as Record<
      string,
      string[]
    >;
    const order = negotiateLanguages([locale], Object.keys(listing), {
      defaultLocale: 'en',
      strategy: 'filtering',
    }).filter((l) => l !== 'en');
    if (listing.en) order.push('en');
    const bundles = await Promise.all(
      order.map(async (l) => {
        const bundle = new FluentBundle(l);
        for (const file of listing[l] ?? []) {
          const text = await (await fetch(`${root}/locales/${l}/${file}`)).text();
          bundle.addResource(new FluentResource(text));
        }
        return bundle;
      }),
    );
    return (id) => {
      for (const bundle of bundles) {
        const pattern = bundle.getMessage(id)?.value;
        if (pattern) return bundle.formatPattern(pattern);
      }
      return id;
    };
  } catch {
    return (id) => id;
  }
}
