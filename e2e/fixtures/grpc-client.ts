import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import { join } from 'node:path';

// Proto path is relative to the e2e directory - go up one level to find proto/
const PROTO_PATH = join(process.cwd(), '../proto/centy/v1/centy.proto');

// Load proto definition
const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
  keepCase: false,
  longs: String,
  enums: String,
  defaults: true,
  oneofs: true,
  includeDirs: [join(process.cwd(), '../proto')],
});

const protoDescriptor = grpc.loadPackageDefinition(packageDefinition) as any;

// Type definitions for gRPC client methods
export interface CentyClient {
  // Init
  init(
    request: InitRequest,
    callback: (error: grpc.ServiceError | null, response: InitResponse) => void
  ): void;
  getReconciliationPlan(
    request: GetReconciliationPlanRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ReconciliationPlan
    ) => void
  ): void;
  executeReconciliation(
    request: ExecuteReconciliationRequest,
    callback: (error: grpc.ServiceError | null, response: InitResponse) => void
  ): void;
  isInitialized(
    request: IsInitializedRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: IsInitializedResponse
    ) => void
  ): void;

  // Issues
  createIssue(
    request: CreateIssueRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: CreateIssueResponse
    ) => void
  ): void;
  getIssue(
    request: GetIssueRequest,
    callback: (error: grpc.ServiceError | null, response: GetIssueResponse) => void
  ): void;
  getIssueByDisplayNumber(
    request: GetIssueByDisplayNumberRequest,
    callback: (error: grpc.ServiceError | null, response: GetIssueResponse) => void
  ): void;
  listIssues(
    request: ListIssuesRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ListIssuesResponse
    ) => void
  ): void;
  updateIssue(
    request: UpdateIssueRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: UpdateIssueResponse
    ) => void
  ): void;
  deleteIssue(
    request: DeleteIssueRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: DeleteIssueResponse
    ) => void
  ): void;
  softDeleteIssue(
    request: SoftDeleteIssueRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: SoftDeleteIssueResponse
    ) => void
  ): void;
  restoreIssue(
    request: RestoreIssueRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: RestoreIssueResponse
    ) => void
  ): void;
  getNextIssueNumber(
    request: GetNextIssueNumberRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: GetNextIssueNumberResponse
    ) => void
  ): void;

  // Docs
  createDoc(
    request: CreateDocRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: CreateDocResponse
    ) => void
  ): void;
  getDoc(
    request: GetDocRequest,
    callback: (error: grpc.ServiceError | null, response: GetDocResponse) => void
  ): void;
  listDocs(
    request: ListDocsRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ListDocsResponse
    ) => void
  ): void;
  updateDoc(
    request: UpdateDocRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: UpdateDocResponse
    ) => void
  ): void;
  deleteDoc(
    request: DeleteDocRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: DeleteDocResponse
    ) => void
  ): void;

  // Assets
  addAsset(
    request: AddAssetRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: AddAssetResponse
    ) => void
  ): void;
  listAssets(
    request: ListAssetsRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ListAssetsResponse
    ) => void
  ): void;
  getAsset(
    request: GetAssetRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: GetAssetResponse
    ) => void
  ): void;
  deleteAsset(
    request: DeleteAssetRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: DeleteAssetResponse
    ) => void
  ): void;
  listSharedAssets(
    request: ListSharedAssetsRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ListAssetsResponse
    ) => void
  ): void;

  // Projects
  listProjects(
    request: ListProjectsRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ListProjectsResponse
    ) => void
  ): void;
  registerProject(
    request: RegisterProjectRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: RegisterProjectResponse
    ) => void
  ): void;
  untrackProject(
    request: UntrackProjectRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: UntrackProjectResponse
    ) => void
  ): void;
  getProjectInfo(
    request: GetProjectInfoRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: GetProjectInfoResponse
    ) => void
  ): void;

  // Config
  getConfig(
    request: GetConfigRequest,
    callback: (error: grpc.ServiceError | null, response: GetConfigResponse) => void
  ): void;
  updateConfig(
    request: UpdateConfigRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: UpdateConfigResponse
    ) => void
  ): void;
  getManifest(
    request: GetManifestRequest,
    callback: (error: grpc.ServiceError | null, response: GetManifestResponse) => void
  ): void;

  // Daemon control
  getDaemonInfo(
    request: GetDaemonInfoRequest,
    callback: (error: grpc.ServiceError | null, response: DaemonInfo) => void
  ): void;
  shutdown(
    request: ShutdownRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ShutdownResponse
    ) => void
  ): void;
  restart(
    request: RestartRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: RestartResponse
    ) => void
  ): void;

  // Organizations
  createOrganization(
    request: CreateOrganizationRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: CreateOrganizationResponse
    ) => void
  ): void;
  listOrganizations(
    request: ListOrganizationsRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ListOrganizationsResponse
    ) => void
  ): void;
  getOrganization(
    request: GetOrganizationRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: GetOrganizationResponse
    ) => void
  ): void;
  deleteOrganization(
    request: DeleteOrganizationRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: DeleteOrganizationResponse
    ) => void
  ): void;
  setProjectOrganization(
    request: SetProjectOrganizationRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: SetProjectOrganizationResponse
    ) => void
  ): void;

  // Org Issues
  createOrgIssue(
    request: CreateOrgIssueRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: CreateOrgIssueResponse
    ) => void
  ): void;
  getOrgIssue(
    request: GetOrgIssueRequest,
    callback: (error: grpc.ServiceError | null, response: OrgIssue) => void
  ): void;
  getOrgIssueByDisplayNumber(
    request: GetOrgIssueByDisplayNumberRequest,
    callback: (error: grpc.ServiceError | null, response: OrgIssue) => void
  ): void;
  listOrgIssues(
    request: ListOrgIssuesRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ListOrgIssuesResponse
    ) => void
  ): void;
  updateOrgIssue(
    request: UpdateOrgIssueRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: UpdateOrgIssueResponse
    ) => void
  ): void;
  deleteOrgIssue(
    request: DeleteOrgIssueRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: DeleteOrgIssueResponse
    ) => void
  ): void;
  getOrgConfig(
    request: GetOrgConfigRequest,
    callback: (error: grpc.ServiceError | null, response: OrgConfig) => void
  ): void;
  updateOrgConfig(
    request: UpdateOrgConfigRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: UpdateOrgConfigResponse
    ) => void
  ): void;

  // Links
  createLink(
    request: CreateLinkRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: CreateLinkResponse
    ) => void
  ): void;
  deleteLink(
    request: DeleteLinkRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: DeleteLinkResponse
    ) => void
  ): void;
  listLinks(
    request: ListLinksRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ListLinksResponse
    ) => void
  ): void;
  getAvailableLinkTypes(
    request: GetAvailableLinkTypesRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: GetAvailableLinkTypesResponse
    ) => void
  ): void;

  // Users
  createUser(
    request: CreateUserRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: CreateUserResponse
    ) => void
  ): void;
  getUser(
    request: GetUserRequest,
    callback: (error: grpc.ServiceError | null, response: GetUserResponse) => void
  ): void;
  listUsers(
    request: ListUsersRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ListUsersResponse
    ) => void
  ): void;
  updateUser(
    request: UpdateUserRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: UpdateUserResponse
    ) => void
  ): void;
  deleteUser(
    request: DeleteUserRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: DeleteUserResponse
    ) => void
  ): void;
  syncUsers(
    request: SyncUsersRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: SyncUsersResponse
    ) => void
  ): void;

  // For cleanup
  close(): void;
}

// Request/Response types
export interface InitRequest {
  projectPath: string;
  force?: boolean;
  decisions?: ReconciliationDecisions;
}

export interface InitResponse {
  success: boolean;
  error: string;
  created: string[];
  restored: string[];
  reset: string[];
  skipped: string[];
  manifest?: Manifest;
}

export interface GetReconciliationPlanRequest {
  projectPath: string;
}

export interface ReconciliationPlan {
  toCreate: FileInfo[];
  toRestore: FileInfo[];
  toReset: FileInfo[];
  upToDate: FileInfo[];
  userFiles: FileInfo[];
  needsDecisions: boolean;
}

export interface ExecuteReconciliationRequest {
  projectPath: string;
  decisions: ReconciliationDecisions;
}

export interface ReconciliationDecisions {
  restore: string[];
  reset: string[];
}

export interface IsInitializedRequest {
  projectPath: string;
}

export interface IsInitializedResponse {
  initialized: boolean;
  centyPath: string;
}

export interface CreateIssueRequest {
  projectPath: string;
  title: string;
  description?: string;
  priority?: number;
  status?: string;
  customFields?: Record<string, string>;
  template?: string;
  isOrgIssue?: boolean;
}

export interface OrgSyncResult {
  projectPath: string;
  success: boolean;
  error: string;
}

export interface CreateIssueResponse {
  success: boolean;
  error: string;
  id: string;
  displayNumber: number;
  issueNumber: string;
  createdFiles: string[];
  manifest?: Manifest;
  orgDisplayNumber?: number;
  syncResults?: OrgSyncResult[];
}

export interface GetIssueRequest {
  projectPath: string;
  issueId: string;
}

export interface GetIssueByDisplayNumberRequest {
  projectPath: string;
  displayNumber: number;
}

export interface Issue {
  id: string;
  displayNumber: number;
  issueNumber: string;
  title: string;
  description: string;
  metadata: IssueMetadata;
}

export interface IssueMetadata {
  displayNumber: number;
  status: string;
  priority: number;
  createdAt: string;
  updatedAt: string;
  customFields: Record<string, string>;
  priorityLabel: string;
  draft?: boolean;
  deletedAt?: string;
  isOrgIssue?: boolean;
  orgSlug?: string;
  orgDisplayNumber?: number;
}

export interface GetIssueResponse {
  success: boolean;
  error: string;
  issue: Issue;
}

export interface ListIssuesRequest {
  projectPath: string;
  status?: string;
  priority?: number;
}

export interface ListIssuesResponse {
  issues: Issue[];
  totalCount: number;
}

export interface UpdateIssueRequest {
  projectPath: string;
  issueId: string;
  title?: string;
  description?: string;
  status?: string;
  priority?: number;
  customFields?: Record<string, string>;
}

export interface UpdateIssueResponse {
  success: boolean;
  error: string;
  issue?: Issue;
  manifest?: Manifest;
  syncResults?: OrgSyncResult[];
}

export interface DeleteIssueRequest {
  projectPath: string;
  issueId: string;
}

export interface DeleteIssueResponse {
  success: boolean;
  error: string;
  manifest?: Manifest;
}

export interface SoftDeleteIssueRequest {
  projectPath: string;
  issueId: string;
}

export interface SoftDeleteIssueResponse {
  success: boolean;
  error: string;
  issue?: Issue;
  manifest?: Manifest;
}

export interface RestoreIssueRequest {
  projectPath: string;
  issueId: string;
}

export interface RestoreIssueResponse {
  success: boolean;
  error: string;
  issue?: Issue;
  manifest?: Manifest;
}

export interface GetNextIssueNumberRequest {
  projectPath: string;
}

export interface GetNextIssueNumberResponse {
  issueNumber: string;
}

export interface CreateDocRequest {
  projectPath: string;
  title: string;
  content?: string;
  slug?: string;
  template?: string;
}

export interface CreateDocResponse {
  success: boolean;
  error: string;
  slug: string;
  createdFile: string;
  manifest?: Manifest;
}

export interface GetDocRequest {
  projectPath: string;
  slug: string;
}

export interface Doc {
  slug: string;
  title: string;
  content: string;
  metadata: DocMetadata;
}

export interface DocMetadata {
  createdAt: string;
  updatedAt: string;
}

export interface GetDocResponse {
  success: boolean;
  error: string;
  doc: Doc;
}

export interface ListDocsRequest {
  projectPath: string;
}

export interface ListDocsResponse {
  docs: Doc[];
  totalCount: number;
}

export interface UpdateDocRequest {
  projectPath: string;
  slug: string;
  title?: string;
  content?: string;
  newSlug?: string;
}

export interface UpdateDocResponse {
  success: boolean;
  error: string;
  doc?: Doc;
  manifest?: Manifest;
}

export interface DeleteDocRequest {
  projectPath: string;
  slug: string;
}

export interface DeleteDocResponse {
  success: boolean;
  error: string;
  manifest?: Manifest;
}

export interface AddAssetRequest {
  projectPath: string;
  issueId?: string;
  filename: string;
  data: Buffer;
  isShared?: boolean;
}

export interface AddAssetResponse {
  success: boolean;
  error: string;
  asset?: Asset;
  path: string;
  manifest?: Manifest;
}

export interface Asset {
  filename: string;
  hash: string;
  size: number;
  mimeType: string;
  isShared: boolean;
  createdAt: string;
}

export interface ListAssetsRequest {
  projectPath: string;
  issueId?: string;
  includeShared?: boolean;
}

export interface ListAssetsResponse {
  assets: Asset[];
  totalCount: number;
}

export interface GetAssetRequest {
  projectPath: string;
  issueId?: string;
  filename: string;
  isShared?: boolean;
}

export interface GetAssetResponse {
  success: boolean;
  error: string;
  data: Buffer;
  asset?: Asset;
}

export interface DeleteAssetRequest {
  projectPath: string;
  issueId?: string;
  filename: string;
  isShared?: boolean;
}

export interface DeleteAssetResponse {
  success: boolean;
  error: string;
  filename: string;
  wasShared: boolean;
  manifest?: Manifest;
}

export interface ListSharedAssetsRequest {
  projectPath: string;
}

export interface ListProjectsRequest {
  includeStale?: boolean;
}

export interface ListProjectsResponse {
  projects: ProjectInfo[];
  totalCount: number;
}

export interface ProjectInfo {
  path: string;
  firstAccessed: string;
  lastAccessed: string;
  issueCount: number;
  docCount: number;
  initialized: boolean;
  name: string;
}

export interface RegisterProjectRequest {
  projectPath: string;
}

export interface RegisterProjectResponse {
  success: boolean;
  error: string;
  project?: ProjectInfo;
}

export interface UntrackProjectRequest {
  projectPath: string;
}

export interface UntrackProjectResponse {
  success: boolean;
  error: string;
}

export interface GetProjectInfoRequest {
  projectPath: string;
}

export interface GetProjectInfoResponse {
  found: boolean;
  project?: ProjectInfo;
}

export interface GetConfigRequest {
  projectPath: string;
}

export interface GetConfigResponse {
  success: boolean;
  error: string;
  config: Config;
}

export interface Config {
  customFields: CustomFieldDefinition[];
  defaults: Record<string, string>;
  priorityLevels: number;
  version: string;
  stateColors: Record<string, string>;
  priorityColors: Record<string, string>;
}

export interface CustomFieldDefinition {
  name: string;
  fieldType: string;
  required: boolean;
  defaultValue: string;
  enumValues: string[];
}

export interface UpdateConfigRequest {
  projectPath: string;
  config: Partial<Config>;
}

export interface UpdateConfigResponse {
  success: boolean;
  error: string;
  config?: Config;
}

export interface GetManifestRequest {
  projectPath: string;
}

export interface Manifest {
  schemaVersion: number;
  centyVersion: string;
  createdAt: string;
  updatedAt: string;
}

export interface GetManifestResponse {
  success: boolean;
  error: string;
  manifest: Manifest;
}

export interface FileInfo {
  path: string;
  fileType: string;
  hash: string;
  contentPreview: string;
}

export interface GetDaemonInfoRequest {}

export interface DaemonInfo {
  version: string;
  binaryPath: string;
  vscodeAvailable: boolean;
}

export interface ShutdownRequest {
  delaySeconds?: number;
}

export interface ShutdownResponse {
  success: boolean;
  message: string;
}

export interface RestartRequest {
  delaySeconds?: number;
}

export interface RestartResponse {
  success: boolean;
  message: string;
}

// ============ Organization Types ============

export interface Organization {
  slug: string;
  name: string;
  description: string;
  createdAt: string;
  updatedAt: string;
  projectCount: number;
}

export interface CreateOrganizationRequest {
  slug?: string;
  name: string;
  description?: string;
}

export interface CreateOrganizationResponse {
  success: boolean;
  error: string;
  organization?: Organization;
}

export interface ListOrganizationsRequest {}

export interface ListOrganizationsResponse {
  organizations: Organization[];
  totalCount: number;
}

export interface GetOrganizationRequest {
  slug: string;
}

export interface GetOrganizationResponse {
  found: boolean;
  organization?: Organization;
}

export interface DeleteOrganizationRequest {
  slug: string;
}

export interface DeleteOrganizationResponse {
  success: boolean;
  error: string;
  unassignedProjects: number;
}

export interface SetProjectOrganizationRequest {
  projectPath: string;
  organizationSlug: string;
}

export interface SetProjectOrganizationResponse {
  success: boolean;
  error: string;
}

// ============ Org Issue Types ============

export interface OrgIssue {
  id: string;
  displayNumber: number;
  title: string;
  description: string;
  metadata: OrgIssueMetadata;
}

export interface OrgIssueMetadata {
  displayNumber: number;
  status: string;
  priority: number;
  createdAt: string;
  updatedAt: string;
  customFields: Record<string, string>;
  priorityLabel: string;
  referencedProjects: string[];
}

export interface CreateOrgIssueRequest {
  organizationSlug: string;
  title: string;
  description?: string;
  priority?: number;
  status?: string;
  customFields?: Record<string, string>;
  referencedProjects?: string[];
}

export interface CreateOrgIssueResponse {
  success: boolean;
  error: string;
  id: string;
  displayNumber: number;
  createdFiles: string[];
}

export interface GetOrgIssueRequest {
  organizationSlug: string;
  issueId: string;
}

export interface GetOrgIssueByDisplayNumberRequest {
  organizationSlug: string;
  displayNumber: number;
}

export interface ListOrgIssuesRequest {
  organizationSlug: string;
  status?: string;
  priority?: number;
  referencedProject?: string;
}

export interface ListOrgIssuesResponse {
  issues: OrgIssue[];
  totalCount: number;
}

export interface UpdateOrgIssueRequest {
  organizationSlug: string;
  issueId: string;
  title?: string;
  description?: string;
  status?: string;
  priority?: number;
  customFields?: Record<string, string>;
  addReferencedProjects?: string[];
  removeReferencedProjects?: string[];
}

export interface UpdateOrgIssueResponse {
  success: boolean;
  error: string;
  issue?: OrgIssue;
}

export interface DeleteOrgIssueRequest {
  organizationSlug: string;
  issueId: string;
}

export interface DeleteOrgIssueResponse {
  success: boolean;
  error: string;
}

export interface OrgConfig {
  priorityLevels: number;
  customFields: CustomFieldDefinition[];
}

export interface GetOrgConfigRequest {
  organizationSlug: string;
}

export interface UpdateOrgConfigRequest {
  organizationSlug: string;
  config: Partial<OrgConfig>;
}

export interface UpdateOrgConfigResponse {
  success: boolean;
  error: string;
  config?: OrgConfig;
}

// ============ Link Types ============

export enum LinkTargetType {
  UNSPECIFIED = 0,
  ISSUE = 1,
  DOC = 2,
}

export interface Link {
  targetId: string;
  targetType: LinkTargetType | string;
  linkType: string;
  createdAt: string;
}

export interface CreateLinkRequest {
  projectPath: string;
  sourceId: string;
  sourceType: LinkTargetType | string;
  targetId: string;
  targetType: LinkTargetType | string;
  linkType: string;
}

export interface CreateLinkResponse {
  success: boolean;
  error: string;
  createdLink?: Link;
  inverseLink?: Link;
}

export interface DeleteLinkRequest {
  projectPath: string;
  sourceId: string;
  sourceType: LinkTargetType | string;
  targetId: string;
  targetType: LinkTargetType | string;
  linkType?: string;
}

export interface DeleteLinkResponse {
  success: boolean;
  error: string;
  deletedCount: number;
}

export interface ListLinksRequest {
  projectPath: string;
  entityId: string;
  entityType: LinkTargetType | string;
}

export interface ListLinksResponse {
  links: Link[];
  totalCount: number;
}

export interface GetAvailableLinkTypesRequest {
  projectPath: string;
}

export interface LinkTypeInfo {
  name: string;
  inverse: string;
  description: string;
  isBuiltin: boolean;
}

export interface GetAvailableLinkTypesResponse {
  linkTypes: LinkTypeInfo[];
}

// ============ User Types ============

export interface User {
  id: string;
  name: string;
  email: string;
  gitUsernames: string[];
  createdAt: string;
  updatedAt: string;
}

export interface GetUserResponse {
  success: boolean;
  error: string;
  user: User;
}

export interface CreateUserRequest {
  projectPath: string;
  id: string;
  name: string;
  email?: string;
  gitUsernames?: string[];
}

export interface CreateUserResponse {
  success: boolean;
  error: string;
  user?: User;
  manifest?: Manifest;
}

export interface GetUserRequest {
  projectPath: string;
  userId: string;
}

export interface ListUsersRequest {
  projectPath: string;
  gitUsername?: string;
}

export interface ListUsersResponse {
  users: User[];
  totalCount: number;
}

export interface UpdateUserRequest {
  projectPath: string;
  userId: string;
  name?: string;
  email?: string;
  gitUsernames?: string[];
}

export interface UpdateUserResponse {
  success: boolean;
  error: string;
  user?: User;
  manifest?: Manifest;
}

export interface DeleteUserRequest {
  projectPath: string;
  userId: string;
}

export interface DeleteUserResponse {
  success: boolean;
  error: string;
  manifest?: Manifest;
}

export interface GitContributor {
  name: string;
  email: string;
}

export interface SyncUsersRequest {
  projectPath: string;
  dryRun: boolean;
}

export interface SyncUsersResponse {
  success: boolean;
  error: string;
  created: string[];
  skipped: string[];
  errors: string[];
  wouldCreate: GitContributor[];
  wouldSkip: GitContributor[];
  manifest?: Manifest;
}

/**
 * Create a gRPC client for the Centy daemon.
 * Uses plain text (insecure) transport for testing.
 */
export function createGrpcClient(
  address: string = '127.0.0.1:50051'
): CentyClient {
  const CentyDaemon = protoDescriptor.centy.v1.CentyDaemon;

  const client = new CentyDaemon(
    address,
    grpc.credentials.createInsecure()
  ) as CentyClient;

  return client;
}

/**
 * Promisified wrapper for gRPC client methods.
 */
export function promisifyClient(client: CentyClient) {
  const promisify =
    <TReq, TRes>(method: (req: TReq, cb: (err: any, res: TRes) => void) => void) =>
    (request: TReq): Promise<TRes> =>
      new Promise((resolve, reject) => {
        method.call(client, request, (err: any, response: TRes) => {
          if (err) reject(err);
          else resolve(response);
        });
      });

  return {
    // Init
    init: promisify<InitRequest, InitResponse>(client.init),
    getReconciliationPlan: promisify<GetReconciliationPlanRequest, ReconciliationPlan>(
      client.getReconciliationPlan
    ),
    executeReconciliation: promisify<ExecuteReconciliationRequest, InitResponse>(
      client.executeReconciliation
    ),
    isInitialized: promisify<IsInitializedRequest, IsInitializedResponse>(
      client.isInitialized
    ),

    // Issues
    createIssue: promisify<CreateIssueRequest, CreateIssueResponse>(client.createIssue),
    getIssue: (request: GetIssueRequest): Promise<Issue> =>
      promisify<GetIssueRequest, GetIssueResponse>(client.getIssue)(request).then((r) => r.issue),
    getIssueByDisplayNumber: (request: GetIssueByDisplayNumberRequest): Promise<Issue> =>
      promisify<GetIssueByDisplayNumberRequest, GetIssueResponse>(client.getIssueByDisplayNumber)(request).then((r) => r.issue),
    listIssues: promisify<ListIssuesRequest, ListIssuesResponse>(client.listIssues),
    updateIssue: promisify<UpdateIssueRequest, UpdateIssueResponse>(client.updateIssue),
    deleteIssue: promisify<DeleteIssueRequest, DeleteIssueResponse>(client.deleteIssue),
    softDeleteIssue: promisify<SoftDeleteIssueRequest, SoftDeleteIssueResponse>(client.softDeleteIssue),
    restoreIssue: promisify<RestoreIssueRequest, RestoreIssueResponse>(client.restoreIssue),
    getNextIssueNumber: promisify<GetNextIssueNumberRequest, GetNextIssueNumberResponse>(
      client.getNextIssueNumber
    ),

    // Docs
    createDoc: promisify<CreateDocRequest, CreateDocResponse>(client.createDoc),
    getDoc: (request: GetDocRequest): Promise<Doc> =>
      promisify<GetDocRequest, GetDocResponse>(client.getDoc)(request).then((r) => r.doc),
    listDocs: promisify<ListDocsRequest, ListDocsResponse>(client.listDocs),
    updateDoc: promisify<UpdateDocRequest, UpdateDocResponse>(client.updateDoc),
    deleteDoc: promisify<DeleteDocRequest, DeleteDocResponse>(client.deleteDoc),

    // Assets
    addAsset: promisify<AddAssetRequest, AddAssetResponse>(client.addAsset),
    listAssets: promisify<ListAssetsRequest, ListAssetsResponse>(client.listAssets),
    getAsset: promisify<GetAssetRequest, GetAssetResponse>(client.getAsset),
    deleteAsset: promisify<DeleteAssetRequest, DeleteAssetResponse>(client.deleteAsset),
    listSharedAssets: promisify<ListSharedAssetsRequest, ListAssetsResponse>(
      client.listSharedAssets
    ),

    // Projects
    listProjects: promisify<ListProjectsRequest, ListProjectsResponse>(client.listProjects),
    registerProject: promisify<RegisterProjectRequest, RegisterProjectResponse>(
      client.registerProject
    ),
    untrackProject: promisify<UntrackProjectRequest, UntrackProjectResponse>(
      client.untrackProject
    ),
    getProjectInfo: promisify<GetProjectInfoRequest, GetProjectInfoResponse>(
      client.getProjectInfo
    ),

    // Config
    getConfig: (request: GetConfigRequest): Promise<Config> =>
      promisify<GetConfigRequest, GetConfigResponse>(client.getConfig)(request).then((r) => r.config),
    updateConfig: promisify<UpdateConfigRequest, UpdateConfigResponse>(client.updateConfig),
    getManifest: (request: GetManifestRequest): Promise<Manifest> =>
      promisify<GetManifestRequest, GetManifestResponse>(client.getManifest)(request).then((r) => r.manifest),

    // Daemon control
    getDaemonInfo: promisify<GetDaemonInfoRequest, DaemonInfo>(client.getDaemonInfo),
    shutdown: promisify<ShutdownRequest, ShutdownResponse>(client.shutdown),
    restart: promisify<RestartRequest, RestartResponse>(client.restart),

    // Organizations
    createOrganization: promisify<CreateOrganizationRequest, CreateOrganizationResponse>(
      client.createOrganization
    ),
    listOrganizations: promisify<ListOrganizationsRequest, ListOrganizationsResponse>(
      client.listOrganizations
    ),
    getOrganization: promisify<GetOrganizationRequest, GetOrganizationResponse>(
      client.getOrganization
    ),
    deleteOrganization: promisify<DeleteOrganizationRequest, DeleteOrganizationResponse>(
      client.deleteOrganization
    ),
    setProjectOrganization: promisify<SetProjectOrganizationRequest, SetProjectOrganizationResponse>(
      client.setProjectOrganization
    ),

    // Org Issues
    createOrgIssue: promisify<CreateOrgIssueRequest, CreateOrgIssueResponse>(
      client.createOrgIssue
    ),
    getOrgIssue: promisify<GetOrgIssueRequest, OrgIssue>(client.getOrgIssue),
    getOrgIssueByDisplayNumber: promisify<GetOrgIssueByDisplayNumberRequest, OrgIssue>(
      client.getOrgIssueByDisplayNumber
    ),
    listOrgIssues: promisify<ListOrgIssuesRequest, ListOrgIssuesResponse>(
      client.listOrgIssues
    ),
    updateOrgIssue: promisify<UpdateOrgIssueRequest, UpdateOrgIssueResponse>(
      client.updateOrgIssue
    ),
    deleteOrgIssue: promisify<DeleteOrgIssueRequest, DeleteOrgIssueResponse>(
      client.deleteOrgIssue
    ),
    getOrgConfig: promisify<GetOrgConfigRequest, OrgConfig>(client.getOrgConfig),
    updateOrgConfig: promisify<UpdateOrgConfigRequest, UpdateOrgConfigResponse>(
      client.updateOrgConfig
    ),

    // Links
    createLink: promisify<CreateLinkRequest, CreateLinkResponse>(client.createLink),
    deleteLink: promisify<DeleteLinkRequest, DeleteLinkResponse>(client.deleteLink),
    listLinks: promisify<ListLinksRequest, ListLinksResponse>(client.listLinks),
    getAvailableLinkTypes: promisify<GetAvailableLinkTypesRequest, GetAvailableLinkTypesResponse>(
      client.getAvailableLinkTypes
    ),

    // Users
    createUser: promisify<CreateUserRequest, CreateUserResponse>(client.createUser),
    getUser: (request: GetUserRequest): Promise<User> =>
      promisify<GetUserRequest, GetUserResponse>(client.getUser)(request).then((r) => r.user),
    listUsers: promisify<ListUsersRequest, ListUsersResponse>(client.listUsers),
    updateUser: promisify<UpdateUserRequest, UpdateUserResponse>(client.updateUser),
    deleteUser: promisify<DeleteUserRequest, DeleteUserResponse>(client.deleteUser),
    syncUsers: promisify<SyncUsersRequest, SyncUsersResponse>(client.syncUsers),

    // Close connection
    close: () => client.close(),
  };
}

export type PromisifiedCentyClient = ReturnType<typeof promisifyClient>;
