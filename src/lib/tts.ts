const Credential = require('@alicloud/credentials');
const OpenApi = require('@alicloud/openapi-client');
const Util = require('@alicloud/tea-util');


export default class TTS {
  public static async AliyunAuth(ak: string, sk: string): Promise<string> {
    const credentialsConfig = new Credential.Config({
      // 凭证类型。
      type: 'access_key',
      // 设置为AccessKey ID值。
      accessKeyId: ak,
      // 设置为AccessKey Secret值。
      accessKeySecret: sk,
    })

    const credentialClient = new Credential.default(credentialsConfig)

    let config = new OpenApi.Config({
      credential: credentialClient,
    })

    config.endpoint = `nls-meta.cn-shanghai.aliyuncs.com`

    const client = new OpenApi.default(config)

    let params = new OpenApi.Params({
      action: 'CreateToken',
      version: '2019-02-28',
      protocol: 'HTTPS',
      method: 'POST',
      authType: 'AK',
      style: 'RPC',
      pathname: `/`,
      reqBodyType: 'json',
      bodyType: 'json',
    })

    let runtime = new Util.RuntimeOptions({ });
    let request = new OpenApi.OpenApiRequest({ });

    const result = await client.callApi(params, request, runtime);

    return result.body.Token.Id
  }

  public static async Aliyun(
    text: string,
    endpoint: string,
    appkey: string,
    token: string
  ): Promise<Uint8Array | null> {
    const url = `${endpoint}`
    const params = JSON.stringify({
      appkey: appkey,
      text: text,
      token: token,
      format: 'mp3',
    })

    const options = {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: params,
    }

    try {
      const response = await fetch(url, options)
      if (!response.ok) {
        throw new Error(
          `HTTP error! status: ${response.status} ${await response.text()} ${params}`
        )
      }
      return await response.bytes()
    } catch (error) {
      console.error(`TTS Aliyun failed: ${error}`)
      return null
    }
  }

  public static async Custom(
    text: string,
    endpoint: string,
    token: string
  ): Promise<Uint8Array | null> {
    const url = `http://${endpoint}`
    const params = new URLSearchParams()
    params.append('text', text)
    params.append('token', token)

    // GET endpoint, params in query string
    try {
      const response = await fetch(`${url}?${params.toString()}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
      })
      if (!response.ok) {
        throw new Error(
          `HTTP error! status: ${response.status} ${await response.text()}`
        )
      }
      return await response.bytes()
    } catch (error) {
      console.error(`TTS Custom failed: ${error}`)
      return null
    }
  }
}
