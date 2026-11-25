"""Opt-in, local regression wrapper for the official native check-in fixture."""
import argparse
from http.cookies import SimpleCookie
import importlib.util
import json
from pathlib import Path
import sys

def main():
    p=argparse.ArgumentParser(add_help=False)
    p.add_argument('--repo',type=Path,required=True)
    args,rest=p.parse_known_args()
    script=args.repo.resolve()/'ccr-desktop/scripts/checkin_fixture.py'
    spec=importlib.util.spec_from_file_location('official',script)
    official=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(official)
    prepare=official.prepare
    class PreferenceHandler(official.Handler):
        def respond(self):
            account=self.headers.get('new-api-user')
            if account in ('101','103'):
                values=SimpleCookie()
                values.load(self.headers.get('Cookie',''))
                correct=(set(values)=={'consent','session'} and values['consent'].value=='1' and
                         values['session'].value=='synthetic-'+account)
                print('PREF',self.command,official.USERS[account],'ok' if correct else 'mismatch',flush=True)
                # The repository server models each session by an exact one-cookie string.
                # Adapt that stub only after checking the complete two-cookie header.
                if 'Cookie' in self.headers:del self.headers['Cookie']
                self.headers['Cookie']='session=synthetic-'+account if correct else 'fixture-invalid'
            super().respond()
        do_GET=respond
        do_POST=respond
    def with_preference(root,port):
        assert root.resolve().parent==official.OUTPUTS.resolve() and root.name.startswith('checkin-')
        prepare(root,port)
        marker=root/'.ccr-gpui-checkin-fixture'
        assert marker.read_bytes()==b'ccr-gpui-checkin-fixture-v1'
        source=root/'accounts.json'
        doc=json.loads(source.read_text('utf8'))
        for row in doc['accounts']:
            if row['id'] in ('fixture-101','fixture-103'):
                cookies=json.loads(row['cookies_json'])
                cookies['consent']='1'
                row['cookies_json']=json.dumps(cookies)
        source.write_text(json.dumps(doc,indent=2),'utf8')
    official.prepare=with_preference
    official.Handler=PreferenceHandler
    sys.argv=[str(script),*rest]
    official.main()
if __name__=='__main__':main()

