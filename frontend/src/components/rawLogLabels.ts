export function formatInputChannelLabel(inputChannel: string) {
  switch (inputChannel) {
    case 'web':
      return 'Web'
    case 'mobile':
      return 'Mobile'
    case 'cli':
      return 'CLI'
    case 'api':
      return 'API'
    case 'import':
      return 'Import'
    case 'telegram':
      return 'Telegram'
    case 'feishu':
      return 'Feishu'
    case 'wechat_bridge':
      return 'WeChat Bridge'
    default:
      return inputChannel
  }
}

export function formatSourceTypeLabel(sourceType: string) {
  switch (sourceType) {
    case 'manual':
      return 'Manual'
    case 'imported':
      return 'Imported'
    case 'synced':
      return 'Synced'
    default:
      return sourceType
  }
}
