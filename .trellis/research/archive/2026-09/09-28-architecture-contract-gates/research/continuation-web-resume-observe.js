globalThis.ccrWebErrors=[];
page.on('pageerror',error=>ccrWebErrors.push(String(error)));
return {url:page.url(),state:await tabbit.observe({frames:'none',maxChars:5200}),fixture:await page.evaluate(()=>globalThis.ccrFixture ? {unknown:ccrFixture.unknown,settings:ccrFixture.settings,calls:ccrFixture.calls.slice(-8)} : null)};