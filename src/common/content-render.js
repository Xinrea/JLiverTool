// renderContent renders string to html node
function renderContent(content_string) {
    // Match string with regex '[bB][vV][0-9a-zA-Z]*'
    let segments = content_string.split(/([bB][vV][0-9a-zA-Z]+)/)
    console.log(segments)
    // Create parent node
    let node = document.createElement('span')
    node.className = 'content'
    // For every segment
    for (const segment of segments) {
        console.log('render: '+segment)
        if (segment.length === 0) continue
        if (/([bB][vV][0-9a-zA-Z]+)/.test(segment)) {
            // This segment is bv link
            console.log(segment+' is bv link')
            let a = document.createElement('a')
            a.className = 'bv-link'
            let url = 'https://www.bilibili.com/video/'+segment
            a.addEventListener('click', () => {
                window.electron.send('openURL',url)
            })
            a.innerText = segment
            node.append(a)
        } else {
            // Plain text
            let p = document.createElement('span')
            p.innerText = segment
            node.append(p)
        }
    }
    return node
}