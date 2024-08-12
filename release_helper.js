// get release list from github api
// and then write it as a simple html file
// https://api.github.com/repos/xinrea/JLiverTool/releases

const https = require('https');
const fs = require('fs');

const options = {
  hostname: 'api.github.com',
  port: 443,
  path: '/repos/xinrea/JLiverTool/releases',
  method: 'GET',
  headers: {
    'User-Agent': 'Node.js'
  }
};

const req = https.request(options, (res) => {
  let data = '';
  res.on('data', (d) => {
    data += d;
  });
  res.on('end', () => {
    const releases = JSON.parse(data);
    // list all releases with assets and desc
    // set title & language
    let html = '<html lang="zh-CN"><head><meta charset="UTF-8"><title>JLiverTool Releases</title></head><body>';

    html += '<h1>JLiverTool Releases</h1>';
    // contact me
    html += '<p>如有问题请联系 <a href="https://space.bilibili.com/475210">Xinrea</a></p>';
    releases.forEach((release) => {
      html += `<h2>${release.name}</h2>`;
      // show published date in human readable format (GMT+8, 2024/01/01 08:00:00)
      const published_at = new Date(release.published_at);
      html += `<p>${published_at.toLocaleString('zh-CN', {timeZone: 'Asia/Shanghai'})}</p>`;
      html += `<pre>${release.body}</pre>`;
      html += '<ul>';
      // list all assets except *.blockmap
      release.assets.forEach((asset) => {
        if (!asset.name.endsWith('.blockmap')) {
          // download url need to be constructed as raw.vjoi.cn/jlivertool/ + name
          html += `<li><a href="https://raw.vjoi.cn/jlivertool/${asset.name}">${asset.name}</a></li>`;
        }
      });
      html += '</ul>';
    });
    html += '</body></html>';
    fs.writeFileSync('releases.html', html);
    // invoke scp to upload releases.html to server
    const { exec } = require('child_process');
    exec('scp releases.html jwebsite:/var/www/html/tools/index.html', (err, stdout, stderr) => {
      if (err) {
        console.error(err);
        return;
      }
      console.log(stdout);
    })
  });
})

req.on('error', (e) => {
  console.error(e);
})

req.end();
