"""Cell-constrained design studies. Fixture values, not a product renderer."""
from pathlib import Path
from math import floor
import unicodedata
from PIL import Image, ImageDraw, ImageFont

CW, CH, PAD = 10, 24, 16
BG, INSET, SURFACE = '#1e1e2e', '#181825', '#313244'
TEXT, MUTED, RULE = '#cdd6f4', '#a6adc8', '#585b70'
GOOD, WARN = '#a6e3a1', '#f9e2af'
FONT = ImageFont.truetype('C:/Windows/Fonts/consola.ttf', 18)
BOLD = ImageFont.truetype('C:/Windows/Fonts/consolab.ttf', 18)
CJK = ImageFont.truetype('C:/Windows/Fonts/msyh.ttc', 17)
OUT = Path(__file__).resolve().parent

def width(s):
    return sum(2 if unicodedata.east_asian_width(c) in 'WF' else 1 for c in str(s))

class Panel:
    def __init__(self, image, x, y, w, h, accent, zh=False):
        self.d = ImageDraw.Draw(image)
        self.x, self.y, self.w, self.h = x, y, w, h
        self.accent, self.zh = accent, zh
        self.bottom = 0
        self.clipped = False

    def pos(self, x, y):
        return PAD+(self.x+x)*CW, PAD+(self.y+y)*CH

    def put(self, x, y, s, color=TEXT, bold=False):
        if y >= self.h-1:
            self.clipped = True
            return
        assert x >= 0 and x+width(s) <= self.w-1, (x, s, self.w)
        self.bottom = max(self.bottom, y)
        for char in str(s):
            size = width(char)
            font = CJK if size == 2 else (BOLD if bold else FONT)
            self.d.text(self.pos(x, y), char, font=font, fill=color)
            x += size

    def fill(self, x, y, w, h, color):
        px, py = self.pos(x, y)
        self.d.rectangle((px,py,px+w*CW-1,py+h*CH-1),fill=color)

    def rule(self, y):
        if y >= self.h-1:
            self.clipped = True
            return
        x1,y1=self.pos(2,y)
        x2,_=self.pos(self.w-3,y)
        self.d.line((x1,y1+CH//2,x2,y1+CH//2),fill=RULE)

    def wrap(self, x, y, s, color=MUTED, max_width=None):
        limit=max_width or self.w-x-3
        rows=['']
        for char in s:
            if width(rows[-1])+width(char)>limit:
                rows.append('')
            rows[-1]+=char
        for offset,line in enumerate(rows): self.put(x,y+offset,line,color)
        return len(rows)

    def frame(self):
        x1,y1=self.pos(0,0)
        fitted_height=self.h if self.clipped else min(self.h,self.bottom+3)
        x2,y2=self.pos(self.w-1,fitted_height-1)
        self.d.rectangle((x1,y1+CH//2,x2,y2+CH//2),outline=RULE)

    def track(self, x, y, size, value, maximum):
        # Fractional terminal-block geometry, rounded down to one eighth cell.
        if y>=self.h-1:
            self.clipped=True
            return
        size=max(1,size)
        px,py=self.pos(x,y)
        self.d.rectangle((px,py+7,px+size*CW-1,py+18),fill=SURFACE)
        fill=floor((value/maximum if maximum else 0)*size*8)/8
        if fill>0:
            self.d.rectangle((px,py+7,px+fill*CW-1,py+18),fill=self.accent)

    def finish(self):
        if self.clipped:
            hint='向下滚动查看其余内容' if self.zh else 'Scroll for remaining details'
            self.fill(2,self.h-2,self.w-4,1,BG)
            self.put(3,self.h-2,hint,MUTED)
        self.frame()

def draw_panel(image, platform, accent, model, variant='A', x=93, y=9,
               w=107, h=32, zh=False, empty=False,
               endpoint='https://api.example.invalid/v1'):
    p=Panel(image,x,y,w,h,accent,zh)
    label=lambda en,cn: cn if zh else en
    p.fill(1,1,w-2,2,INSET)
    p.put(3,1,'work',accent,True)
    status=label('Enabled  /  Not current','已启用 / 非当前配置')
    status_x=w-width(status)-4
    p.put(status_x,1,label('Enabled','已启用'),GOOD)
    p.put(status_x+width(label('Enabled','已启用')),1,label('  /  Not current',' / 非当前配置'),MUTED)
    p.put(3,2,label('Daily development','日常开发'),TEXT)
    tags=label('Tags: work, daily','标签: work, daily')
    if w>=74: p.put(w-width(tags)-4,2,tags,MUTED)
    else: p.put(3,3,tags,MUTED)

    def connection(start, col=3, available=None, split=False):
        size=available or w-col-3
        rows=[]
        if platform=='Claude':
            rows=[(label('Model','模型'),model),(label('Sonnet','Sonnet'),'sonnet-example'),
                  (label('Provider','提供商'),'example-provider'),(label('Auth','认证'),'Profile API key')]
        elif platform=='Grok':
            rows=[(label('Model','模型'),model),(label('Backend','后端'),'responses / search on'),
                  (label('Provider','提供商'),'example-provider'),(label('Auth','认证'),'Env: EXAMPLE_API_KEY')]
        else:
            rows=[(label('Model','模型'),model),(label('Provider','提供商'),'example-provider'),
                  (label('Endpoint','端点'),endpoint),
                  (label('Auth','认证'),'Env: EXAMPLE_API_KEY')]
        if platform!='Codex': rows.insert(3,(label('Endpoint','端点'),endpoint))
        yrow=start
        for key,value in rows:
            p.put(col,yrow,key,MUTED)
            value_col=col+11
            count=p.wrap(value_col,yrow,value,accent if key==label('Model','模型') else TEXT,max_width=size-11)
            is_codex_model=platform=='Codex' and key==label('Model','模型')
            inline_reasoning=is_codex_model and size>=65 and width(value)<28
            if inline_reasoning:
                p.put(value_col+width(value)+3,yrow,label('Reasoning: high','推理: high'),MUTED)
            yrow+=count
            if is_codex_model and not inline_reasoning:
                p.put(col,yrow,label('Reasoning','推理'),MUTED)
                p.put(value_col,yrow,'high',TEXT)
                yrow+=1
        return yrow

    metrics=[('Input','输入',59.4,'59.4M'),('Cache read','缓存读取',44.9,'44.9M'),
             ('Cache write','缓存写入',1.0,'1.0M'),('Output','输出',4.2,'4.2M'),
             ('Reasoning','推理输出',0.3,'0.3M')]

    def usage(start,col=3,available=None):
        size=available or w-col-3
        p.put(col,start,label('Provider usage','提供商用量'),accent,True)
        scope=label('All-time / '+platform,'全时段 / '+platform)
        if size>=65: p.put(col+size-width(scope),start,scope,MUTED)
        if empty:
            p.put(col,start+2,label('No usage recorded','暂无用量记录'),MUTED)
            p.wrap(col,start+3,label('example-provider / '+platform,'example-provider / '+platform),max_width=size)
            return start+5
        p.fill(col-1,start+1,size+2,2,INSET)
        if size>=80:
            p.put(col,start+1,'8.6K',TEXT,True)
            p.put(col+6,start+1,label('requests','请求'),MUTED)
            p.put(col+26,start+1,'109.8M',TEXT,True)
            p.put(col+34,start+1,label('tokens','tokens'),MUTED)
            cost=label('$32.60 estimate','$32.60 估算')
            p.put(col+size-width(cost),start+1,cost,WARN,True)
        else:
            p.put(col,start+1,label('8.6K req / 109.8M tokens','8.6K 请求 / 109.8M tokens'),TEXT,True)
            p.put(col,start+2,label('$32.60 estimate','$32.60 估算'),WARN,True)
        note=label('All-time provider / official-equivalent USD','提供商全时段累计 / 官方等价口径 USD')
        n=p.wrap(col,start+3,note,MUTED,max_width=size)
        header=start+3+n+1
        p.put(col,header,label('Token type','Token 类型'),MUTED)
        show_bars=size>=58
        if show_bars:
            p.put(col+18,header,label('Scale: 59.4M','刻度上限: 59.4M'),MUTED)
        count_col=col+size-7
        p.put(count_col,header,label('Count','数量'),MUTED)
        for i,(en,cn,value,display) in enumerate(metrics):
            yy=header+1+i
            p.put(col,yy,label(en,cn),TEXT)
            if show_bars: p.track(col+18,yy,size-29,value,59.4)
            p.put(col+size-width(display),yy,display,TEXT,True)
        return header+7

    def activity(start,col=3,available=None):
        size=available or w-col-3
        heading=label('Recorded activations','启用记录')
        p.put(col,start,heading,accent,True)
        count=label('23 switches','23 次切换')
        if size>=55: p.put(col+width(heading)+3,start,count,MUTED)
        else: p.put(col,start+1,count,MUTED)
        if empty:
            p.put(col,start+2,label('No activation record','暂无启用记录'),MUTED)
            return start+4
        times=['2026-10-12 10:42 -07:00','2026-10-11 18:15 -07:00','2026-10-10 09:04 -07:00']
        for i,stamp in enumerate(times):
            p.put(col,start+2+i,'*' if i==0 else '|',accent if i==0 else RULE)
            p.put(col+3,start+2+i,stamp,TEXT if i==0 else MUTED)
            if i==0 and size>=65: p.put(col+31,start+2+i,label('Last used','上次使用'),MUTED)
        note=label('Last used = successful CCR activation.','上次使用按 CCR 成功启用记录统计。')
        n=p.wrap(col,start+6,note,MUTED,max_width=size)
        return start+6+n

    if variant=='B':
        # Independent configuration and consumption rails, no giant key/value gap.
        left_size=43
        split=51
        connection(5,3,left_size,True)
        activity(15,3,left_size)
        usage(5,55,w-59)
        px,py=p.pos(split,4)
        _,bottom=p.pos(split,min(h-3,p.bottom+1))
        p.d.line((px,py,px,bottom),fill=RULE)
    elif variant=='C':
        end=usage(5)
        p.rule(end)
        end=connection(end+1)
        p.rule(end+1)
        activity(end+2)
    else:
        end=connection(4)
        p.rule(end+1)
        end=usage(end+2)
        p.rule(end)
        activity(end+1)
    p.finish()
    return p

def standalone(name, **kwargs):
    w,h=kwargs.get('w',107),kwargs.get('h',32)
    image=Image.new('RGB',(w*CW+PAD*2,(h+2)*CH+PAD*2),BG)
    draw=ImageDraw.Draw(image)
    draw.text((PAD,PAD),name+' / fixture design',font=FONT,fill=MUTED)
    model=kwargs.pop('model','gpt-example')
    draw_panel(image,'Codex','#89b4fa',model,x=0,y=2,**kwargs)
    path=OUT/(name+'.png')
    image.save(path)
    return path
