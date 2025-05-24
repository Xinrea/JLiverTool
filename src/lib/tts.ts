const ALIYUN_APP = 'rSQR37CUynQS4AR5'

export default class TTS {
  public static async Aliyun(
    text: string,
    endpoint: string,
    token: string
  ): Promise<Uint8Array | null> {
    const url = `${endpoint}`
    const params = JSON.stringify({
      appkey: ALIYUN_APP,
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
          `HTTP error! status: ${response.status} ${await response.text()}`
        )
      }
      return await response.bytes()
    } catch (error) {
      console.error(`TTS Aliyun failed: ${error}`)
      return null
    }
  }
}
