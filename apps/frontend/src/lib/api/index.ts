export { getToken, setToken, clearToken } from './client';

export { register, login, getCurrentUser, deleteUser } from './auth';

export {
	listTracks,
	createTrack,
	getTrack,
	updateTrack,
	deleteTrack,
	listAttachments,
	createAttachment,
	updateAttachment,
	deleteAttachment
} from './tracks';

export {
	listCollections,
	createCollection,
	getCollection,
	updateCollection,
	deleteCollection,
	listCollectionTracks,
	addTrackToCollection,
	removeTrackFromCollection,
	updateTrackPosition,
	reorderTracks
} from './collections';
