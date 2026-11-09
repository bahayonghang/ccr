const dialog = page.getByRole('dialog', {name:'查看原始配置',exact:true});
await expect(dialog).toBeVisible();
return {url:page.url(),dialog:await dialog.innerText(),buttons:await dialog.getByRole('button').allTextContents(),errors:ccrWebErrors};