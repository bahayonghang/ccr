import { describe, expect, it } from 'vitest'
import { toVendorKey } from '@/configs/profileDisplayRecord'

describe('toVendorKey 供应商 canonical key', () => {
  it('大小写规范化到 hostname 小写', () => {
    expect(toVendorKey('HTTPS://API.Example.COM/v1')).toBe('api.example.com')
  })

  it('默认端口与显式默认端口归一，非默认端口保留', () => {
    expect(toVendorKey('https://api.example.com')).toBe('api.example.com')
    expect(toVendorKey('https://api.example.com:443/v1')).toBe('api.example.com')
    expect(toVendorKey('http://api.example.com:80')).toBe('api.example.com')
    expect(toVendorKey('https://api.example.com:8443/v1')).toBe('api.example.com:8443')
  })

  it('丢弃 userinfo', () => {
    expect(toVendorKey('https://user:pass@api.example.com/v1')).toBe('api.example.com')
  })

  it('IPv6 保留方括号', () => {
    expect(toVendorKey('https://[2001:db8::1]/v1')).toBe('[2001:db8::1]')
    expect(toVendorKey('https://[2001:db8::1]:8443/v1')).toBe('[2001:db8::1]:8443')
  })

  it('去掉 hostname 尾点', () => {
    expect(toVendorKey('https://api.example.com./v1')).toBe('api.example.com')
  })

  it('无协议输入补 https 后解析', () => {
    expect(toVendorKey('api.example.com/v1')).toBe('api.example.com')
  })

  it('空值与非字符串返回 null', () => {
    expect(toVendorKey('')).toBeNull()
    expect(toVendorKey('   ')).toBeNull()
    expect(toVendorKey(null)).toBeNull()
    expect(toVendorKey(undefined)).toBeNull()
    expect(toVendorKey(12)).toBeNull()
  })

  it('非法输入返回 null', () => {
    expect(toVendorKey('http://')).toBeNull()
    expect(toVendorKey('://missing-host')).toBeNull()
    expect(toVendorKey('not a url %%')).toBeNull()
  })
})
