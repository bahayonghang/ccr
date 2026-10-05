await page.getByRole('button',{name:'源文件',exact:true}).click();
const dialog=page.getByRole('dialog',{name:'查看原始配置',exact:true});
await expect(dialog).toBeVisible({timeout:15000});
return {text:await dialog.innerText(),buttons:await dialog.getByRole('button').allTextContents(),errors:ccrWebErrors};