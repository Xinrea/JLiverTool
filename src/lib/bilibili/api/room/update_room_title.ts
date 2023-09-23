import BasicResponse from '../basic_response'

export type UpdateRoomTitleResponse = BasicResponse & {
  data: {
    sub_session_key: string
    audit_info: {
      audit_title_reason: string
      update_title: string
      audit_title_status: number
      audit_title: string
    }
  }
}

export default UpdateRoomTitleResponse
