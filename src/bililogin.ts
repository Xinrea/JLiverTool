import * as https from 'https'

export enum QrCodeStatus {
    NeedScan,
    NeedConfirm,
    Success
}

export function GetNewQrCode() {
    return new Promise((resolve, reject) => {
        https.get('https://passport.bilibili.com/x/passport-login/web/qrcode/generate', res => {
            res.on('data', chunk => {
                let resp = JSON.parse(chunk.toString())
                // QrCode image is generated from resp['data']['url']
                // oauthKey is used to check QrCode status
                resolve({
                    url: resp['data']['url'],
                    oauthKey: resp['data']['qrcode_key']
                })
            })
            res.on('error', err => {
                reject(err)
            })
        })
    })
}

export function CheckQrCodeStatus(oauthKey: string) {
    return new Promise((resolve, reject) => {
        let postOptions = {
            hostname: 'passport.bilibili.com',
            path:  '/x/passport-login/web/qrcode/poll?qrcode_key=' + oauthKey,
            method: 'GET'
        }
        let statusReq = https.request(postOptions, res => {
            let dd = ''
            res.on('data', secCheck => {
                dd += secCheck
            })
            res.on('end', () => {
                let resp = JSON.parse(dd)
                if (resp['data']['code'] === 0) {
                    let querystring = require('querystring')
                    let url = resp['data']['url']
                    let params = querystring.parse(url.split('?')[1])
                    resolve({
                        status: QrCodeStatus.Success,
                        cookies: params,
                    })
                } else {
                    if (resp['data']['code'] === 86101) {
                        resolve({
                            status: QrCodeStatus.NeedScan
                        })
                    } else if (resp['data']['code'] === 86090) {
                        resolve({
                            status: QrCodeStatus.NeedConfirm
                        })
                    } else {
                        reject(resp)
                    }
                }
            })
            res.on('error', err => {
                reject(err)
            })
        })
        statusReq.end()
    })
}

function cookiesToString(cookies): string {
    return "SESSDATA=" + encodeURIComponent(cookies.SESSDATA) + "; DedeUserID=" + cookies.DedeUserID + "; DedeUserID_ckMd5=" + cookies.DedeUserID__ckMd5
}

export function Logout(cookies) {
    // https://passport.bilibili.com/login/exit/v2
    return new Promise((resolve, reject) => {
        let postData = 'biliCSRF=' + cookies.bili_jct
        let postOptions = {
            hostname: 'passport.bilibili.com',
            path: '/login/exit/v2',
            method: 'POST',
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded',
                'Content-Length': Buffer.byteLength(postData),
                'cookie': cookiesToString(cookies)
            }
        }
        let statusReq = https.request(postOptions, res => {
            let dd = ''
            res.on('data', secCheck => {
                dd += secCheck
            })
            res.on('end', () => {
                let resp = JSON.parse(dd)
                resolve(resp)
            })
            res.on('error', err => {
                reject(err)
            })
        })
        statusReq.write(postData)
        statusReq.end()
    })
}