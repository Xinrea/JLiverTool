import { GiftType } from './bilibili/api/room/gift_config'
import { EmojiContent, Sender, MergeUserInfo, DmExtraInfo } from './types'

type DynamicEmojiContent = {
  id: number
  package_id: number
  text: string
  url: string
  gif_url: string
  mtime: number
  type: number
  attr: number
  meta: {
    size: number
    alias: string
  }
  flags: {
    unlocked: boolean
  }
  activity: any
  webp_url: string
}

// temporary dynamic emoji map
const dynamicEmojis: DynamicEmojiContent[] = [
  {
    id: 129842,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_跑了]',
    url: 'https://i0.hdslb.com/bfs/garb/99b979beea5441033d869cd713fd30aa227f0793.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/528df074d002fa00bc67f58b94b8332a8c54e9d8.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: '跑了',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/3c6551d38c726273798e1cc1e488a3d0633bad5e.webp',
  },
  {
    id: 129843,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_鞠躬]',
    url: 'https://i0.hdslb.com/bfs/garb/4eb395bbe71146e3287658d365d1063063822fc5.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/2e9d311e35bef3463326f78196e767e0413fc7ae.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: '鞠躬',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/13fc84be85e70ccd8b12acc8e96e49203e96b1a9.webp',
  },
  {
    id: 129844,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_摇你]',
    url: 'https://i0.hdslb.com/bfs/garb/a2b93682e76cdd5e9b470ff595af60143e3c3d63.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/863ecc96044f8b945dbbc75252dbf02c3eac38a2.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: '摇你',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/ec27426348fafa3286412d04e8ea851a7a84ce6e.webp',
  },
  {
    id: 129845,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_愤怒]',
    url: 'https://i0.hdslb.com/bfs/garb/9294fb31121f8669b9a3b213cf306bf9bc89384b.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/3901d600c3d545c1f53d74ac4aab5e0af901712d.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: '愤怒',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/5fbc1bda6a6d73b83e51cba2119ea65d0dc74689.webp',
  },
  {
    id: 129846,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_猴]',
    url: 'https://i0.hdslb.com/bfs/garb/94ce3018cb1c76869b9f3b43594cfa02e08fab74.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/8a3eda02f50e687c484c0504dff19f36f1615179.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: '猴',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/4c7be23a801094ea00452a2da5ce3184336df331.webp',
  },
  {
    id: 129847,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_NO]',
    url: 'https://i0.hdslb.com/bfs/garb/4f04ada30963361131b8d612db34a015d2f36dd2.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/c3ee13de411d25bc45c2408a475d03874efbae16.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: 'NO',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/5d124a80db426608c48c76b27d790c711d39b7b7.webp',
  },
  {
    id: 129848,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_贴贴]',
    url: 'https://i0.hdslb.com/bfs/garb/10917799576f0562566c3434cb1229ed3d92d16a.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/3f016930c11e05eefa9ec05b02691ad513acb7b9.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: '贴贴',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/b02b9ebd543b2ee63573daffa191e8a76b7bb384.webp',
  },
  {
    id: 129849,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_kksk]',
    url: 'https://i0.hdslb.com/bfs/garb/841190d55e1c2549ac04176fe9c766c724af66cc.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/f85c6dc34931801ddd36c7be97dc8b2ca21d7b74.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: 'kksk',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/518c5b00cdb1329ae8e26d50c52369394d7a2394.webp',
  },
  {
    id: 129850,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_这辈子完了]',
    url: 'https://i0.hdslb.com/bfs/garb/b5b06a7bd67ded3275e66e342bc1fcb85a25cd0a.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/9985418f36d589c32486373f183e3e5d2a1a0004.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: '这辈子完了',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/85464aa487412c66ab24543a86218fa82fff7f23.webp',
  },
  {
    id: 129851,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_呆]',
    url: 'https://i0.hdslb.com/bfs/garb/660102364a0658490314a48656d05b930e874f95.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/f867cb98283ee486810b2d7a0fbfd94d75fdf784.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: '呆',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/09a0bd1deaa02a4255dc886642123dfe2606ddb8.webp',
  },
  {
    id: 129852,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_唔唔]',
    url: 'https://i0.hdslb.com/bfs/garb/26a30791f0c1eb10b744faa6f1ea15fbb73b44c1.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/5b7658a1e96860f42687cc20af62da65f212ac49.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: '唔唔',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/c82df0783d3cba6af8cebaa43c1f36b1ef3cc0f4.webp',
  },
  {
    id: 129853,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_啊这]',
    url: 'https://i0.hdslb.com/bfs/garb/c4bca1e6a133f0ee0057b29364e66a654f80be9c.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/bba2b66c62532cfede495f72c0a4df43028f7fc3.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: '啊这',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/03e899fb3c823cf337b50100718247f25955ee4b.webp',
  },
  {
    id: 129854,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_失落]',
    url: 'https://i0.hdslb.com/bfs/garb/03c8fe70ac7aa6e890c5746922d03ba0fea3d763.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/6d1583c89f0edc41607254dcd5276fb52763cb02.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: '失落',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/dc085a527f49bedf897d08de2bb07fa7c779641b.webp',
  },
  {
    id: 129855,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_神气]',
    url: 'https://i0.hdslb.com/bfs/garb/b399d926246d14ab262eb7415ba36a4a555d0025.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/0d746ba3aa2c0cf16f0aaa6a9d45f6f6946da8ca.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: '神气',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/8ed028a87af6c2ef79cdd654f3076d7f7f836514.webp',
  },
  {
    id: 129856,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_怎么这样]',
    url: 'https://i0.hdslb.com/bfs/garb/0bcc693add5d360ec9a954dc14a26307e1bc24f6.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/3c0768b34157ccb5bf3c4a3f7577f4d2add57191.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: '怎么这样',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/7e239e9ee33a8e168df4bba58cd9a18c40d9af45.webp',
  },
  {
    id: 129857,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_尼嘻嘻]',
    url: 'https://i0.hdslb.com/bfs/garb/3029b41f9dfd225330689378f45447611fc43566.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/734e42457ea8bb3e85d77e515c3b1e1a4c938f93.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: '尼嘻嘻',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/278ebc1d82a220a127aa60a10b461dbbc71040c6.webp',
  },
  {
    id: 129858,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_惊]',
    url: 'https://i0.hdslb.com/bfs/garb/40ab6543c28c1873b7b26699fd1d8db47a900a40.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/b1e81604808453f2acff0872fc72c3eae34713a6.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: '惊',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/82f222eebba1160d81244d578dda1c4724276056.webp',
  },
  {
    id: 129859,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_害怕]',
    url: 'https://i0.hdslb.com/bfs/garb/3dc037a94a6bcdb548f010df4912069c0cc68bc7.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/708d8ae0896bdca4244dea2898631860246ccc42.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: '害怕',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/4c00a15ab814a8855e3b7106cdc43c1f0a8fa640.webp',
  },
  {
    id: 129860,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_睡觉]',
    url: 'https://i0.hdslb.com/bfs/garb/f7fa3921093f27dd96818c9088b27688c05a7d80.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/0e42f8ff46493eabeb9233aba42d29de303194f5.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: '睡觉',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/dbec05c6adfd3c56c4fa484b00e4095b4d17cec7.webp',
  },
  {
    id: 129861,
    package_id: 9264,
    text: '[轴伊Joi收藏集动态表情包_爆]',
    url: 'https://i0.hdslb.com/bfs/garb/431f057eecc740c95ebcffa6ae6241b924f00b3a.png',
    gif_url:
      'https://i0.hdslb.com/bfs/garb/a87917f17d6da31a3bcfa45f5d7b970a829b6daf.gif',
    mtime: 1767956400,
    type: 3,
    attr: 0,
    meta: {
      size: 2,
      alias: '爆',
    },
    flags: {
      unlocked: false,
    },
    activity: null,
    webp_url:
      'https://i0.hdslb.com/bfs/garb/b7e50a289001583cd06eb1149304149ba5ccc78b.webp',
  },
]

const dynamicEmojisMapToWebP = new Map<string, string>()
for (const emoji of dynamicEmojis) {
  dynamicEmojisMapToWebP.set(emoji.text, emoji.webp_url)
}

export class DanmuMessage {
  sender: Sender = new Sender()
  content: string
  is_generated: boolean = false
  is_special: boolean = false
  emoji_content: EmojiContent = null
  side_index: number = -1
  reply_uname: string = null

  constructor(body: any, user_info: MergeUserInfo = null) {
    // basic info
    if (user_info) {
      this.side_index = user_info.index
    }
    this.sender.uid = body.info[2][0]
    this.sender.uname = body.info[2][1]
    // TODO maybe need the backend service to offer a face cache
    this.sender.face = ''

    const extra_info = body.info[0][15] as DmExtraInfo
    if (extra_info.show_reply && extra_info.reply_uname != '') {
      this.reply_uname = extra_info.reply_uname
    }

    // medalinfo
    // there is an example of medalinfo (info[3]) and its final html structure.
    //  info[3] = [
    //     29, -- medal_level
    //     "轴芯", -- medal_name
    //     "轴伊Joi_Channel", -- anchor_uname
    //     21484828, -- anchor_roomid
    //     2951253, -- #2d0855 medal_color_text
    //     "",
    //     0,
    //     6809855, -- #6718ff medal_color_border
    //     2951253, -- #2d0855 medal_color_start
    //     10329087, -- #9d9bff medal_color_end
    //     3, -- guard_level
    //     1, -- is_lighted
    //     61639371 -- anchor_id (not used)
    //  ],
    // <div class="fans-medal-item" stype="border-color: #67e8ff">
    // <div class="fans-medal-label"
    //   style="background-image: -o-linear-gradient(45deg, #2d0855, #9d9bff); \
    //          background-image: -moz-linear-gradient(45deg, #2d0855, #9d9bff); \
    //          background-image: -webkit-linear-gradient(45deg, #2d0855, #9d9bff);
    //          background-image: linear-gradient(45deg, #2d0855, #9d9bff);">
    //   <i class="medal-deco  medal-guard" style="background-image: url(https://i0.hdslb.com/bfs/live/143f5ec3003b4080d1b5f817a9efdca46d631945.png@44w_44h.webp);"></i>
    //   <span class="fans-medal-content">轴芯</span>
    // </div>
    if (body.info[3]) {
      this.sender.medal_info.anchor_roomid = body.info[3][3]
      this.sender.medal_info.anchor_uname = body.info[3][2]
      this.sender.medal_info.medal_name = body.info[3][1]
      this.sender.medal_info.medal_level = body.info[3][0]

      this.sender.medal_info.medal_color = body.info[3][4]
      this.sender.medal_info.medal_color_border = body.info[3][7]
      this.sender.medal_info.medal_color_start = body.info[3][8]
      this.sender.medal_info.medal_color_end = body.info[3][9]

      // TODO need confirm
      this.sender.medal_info.guard_level = body.info[3][10]
      this.sender.medal_info.is_lighted = body.info[3][11]
    }

    // trim the content and remove line break
    this.content = body.info[1].trim().replace(/[\r\n]/g, '')

    if (body.info[0][12] == 1) {
      console.log('emoji_content', body.info[0][13])
      let emoji_content = body.info[0][13]
      const emoji_key = emoji_content.emoticon_unique?.replace('upower_', '')
      if (emoji_key && dynamicEmojisMapToWebP.has(emoji_key)) {
        console.log(
          'dynamicEmojisMapToWebP',
          dynamicEmojisMapToWebP.get(emoji_key)
        )
        emoji_content.url = dynamicEmojisMapToWebP.get(emoji_key)
      }
      this.emoji_content = emoji_content
    }

    // generated by gift or other kind of activities
    if (body.info[0][9] > 0) {
      this.is_generated = true
    }

    // send by room admin or other special users
    if (body.info[2][2] > 0) {
      this.is_special = true
    }
  }
}

export class GiftMessage {
  id: string
  room: number
  gift_info: GiftType
  sender: Sender
  action: string
  num: number
  timestamp: number
}

export class GuardMessage {
  id: string
  room: number
  sender: Sender
  num: number
  unit: string
  guard_level: number
  price: number
  timestamp: number
}

export class SuperChatMessage {
  id: string
  room: number
  sender: Sender
  message: string
  price: number
  timestamp: number
}

export class InteractMessage {
  sender: Sender
  action: number
}

export class EntryEffectMessage {
  sender: Sender
  privilege_type: number
}

export class GiftInitData {
  gifts: GiftMessage[]
  guards: GuardMessage[]
}
