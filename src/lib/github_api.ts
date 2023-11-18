// Get latest release from github

import axios from 'axios'

export default class GithubApi {
  static async GetLatestRelease() {
    const resp = await axios.get(
      'https://api.github.com/repos/xinrea/jlivertool/releases/latest'
    )
    return resp.data
  }
}
