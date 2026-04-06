
ç
datanest.protodatanest.v1"h
Tag
tag_id (	RtagId
name (	Rname
user_id (	RuserId

created_at (R	createdAt"º
NoteSummary
note_id (	RnoteId
title (	Rtitle'
content_preview (	RcontentPreview
user_id (	RuserId

created_at (R	createdAt

updated_at (R	updatedAt"“

NoteDetail
note_id (	RnoteId
title (	Rtitle
content (	Rcontent
user_id (	RuserId

created_at (R	createdAt

updated_at (R	updatedAt$
tags (2.datanest.v1.TagRtags"C
CreateNoteRequest
title (	Rtitle
content (	Rcontent")
GetNoteRequest
note_id (	RnoteId"
ListNotesRequest"*
SearchNotesRequest
query (	Rquery"C
ListNotesResponse.
notes (2.datanest.v1.NoteSummaryRnotes"&
CreateTagRequest
name (	Rname"
ListTagsRequest"8
ListTagsResponse$
tags (2.datanest.v1.TagRtags"H
AttachTagToNoteRequest
note_id (	RnoteId
tag_id (	RtagId2ç
DataNestServiceE

CreateNote.datanest.v1.CreateNoteRequest.datanest.v1.NoteDetail?
GetNote.datanest.v1.GetNoteRequest.datanest.v1.NoteDetailJ
	ListNotes.datanest.v1.ListNotesRequest.datanest.v1.ListNotesResponseN
SearchNotes.datanest.v1.SearchNotesRequest.datanest.v1.ListNotesResponse<
	CreateTag.datanest.v1.CreateTagRequest.datanest.v1.TagG
ListTags.datanest.v1.ListTagsRequest.datanest.v1.ListTagsResponseO
AttachTagToNote#.datanest.v1.AttachTagToNoteRequest.datanest.v1.NoteDetailB$Z"datanest/go-service/internal/pb;pbbproto3