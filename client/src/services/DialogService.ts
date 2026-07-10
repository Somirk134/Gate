import type { EventBus } from '@/events/EventBus'
import type { AppEventMap, DialogPayload } from '@/types/application'
import { createId } from '@/utils/id'

export interface DialogService {
  show(dialog: DialogPayload): Promise<unknown>
  alert(title: string, content?: string): Promise<unknown>
  confirm(title: string, content?: string): Promise<unknown>
}

export class EventDialogService implements DialogService {
  constructor(private readonly events: EventBus<AppEventMap>) {}

  async show(dialog: DialogPayload) {
    const id = dialog.id ?? createId('dialog')

    await this.events.publish('dialog:show', {
      ...dialog,
      id,
    })

    return id
  }

  alert(title: string, content?: string) {
    return this.show({ type: 'alert', title, content })
  }

  confirm(title: string, content?: string) {
    return this.show({ type: 'confirm', title, content })
  }
}
