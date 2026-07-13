// -- Auth 

export interface User {
  id: string
  email: string
  profile_id: string
  roles: string[]
  is_otp_enabled: boolean
}

export interface AuthResponse {
  access_token: string
  token_type: string
  expires_in: number
  user: User
}

export interface RefreshResponse {
  access_token: string
  token_type: string
  expires_in: number
}

export interface SessionResponse {
  access_token: string
  token_type: string
  expires_in: number
  user: User
}

export interface LoginPayload {
  email: string
  password: string
  otp_code?: string
}

export interface RegisterPayload {
  email: string
  password: string
  password_confirmation: string
}

// -- Profile 

export interface Profile {
  id: string
  user_id: string
  first_name?: string
  last_name?: string
  display_name?: string
  slug?: string
  bio?: string
  avatar_url?: string
  cover_url?: string
  birthday?: string
  age_verified: boolean
  country?: string
  state?: string
  city?: string
  social_network: SocialNetwork
  is_creator: boolean
  is_agency: boolean
  status: 'active' | 'suspended' | 'banned' | 'pending_kyc'
  created_at: string
  updated_at: string
}

export interface PublicProfile {
  id: string
  display_name?: string
  slug?: string
  bio?: string
  avatar_url?: string
  cover_url?: string
  social_network: SocialNetwork
  is_creator: boolean
  age_verified: boolean
  country?: string
}

export interface SocialNetwork {
  instagram?: string
  twitter?: string
  tiktok?: string
  telegram?: string
  website?: string
}

export interface UpdateProfilePayload {
  first_name?: string
  last_name?: string
  display_name?: string
  slug?: string
  bio?: string
  birthday?: string
  country?: string
  state?: string
  city?: string
  social_network?: SocialNetwork
}

// -- Post 

export interface Post {
  id: string
  creator_profile_id: string
  creator_slug?: string
  body?: string
  is_ppv: boolean
  ppv_price_cents?: number
  visibility: 'public' | 'subscribers_only' | 'ppv'
  is_pinned: boolean
  published_at?: string
  likes_count: number
  views_count: number
  created_at: string
  is_locked: boolean
  age_verified?: boolean
  media: MediaFile[]
}

export interface CreatePostPayload {
  body?: string
  is_ppv: boolean
  ppv_price_cents?: number
  visibility: 'public' | 'subscribers_only' | 'ppv'
  scheduled_at?: string
  media_ids: string[]
}

// -- Media 

export interface MediaFile {
  id: string
  media_type: 'video' | 'photo' | 'audio'
  status: 'uploading' | 'processing' | 'ready' | 'failed'
  duration_seconds?: number
  width?: number
  height?: number
  signed_url?: string
  hls_url?: string
  thumbnail_url?: string
}

export interface UploadUrlResponse {
  media_file_id: string
  upload_url: string
  auth_token: string
  object_key: string
}

// -- Subscription 

export interface SubscriptionPlan {
  id: string
  creator_profile_id: string
  name: string
  description?: string
  price_cents: number
  interval_months: number
  discount_percent?: number
  is_active: boolean
}

export interface Subscription {
  id: string
  subscriber_profile_id: string
  creator_profile_id: string
  plan_id?: string
  status: 'active' | 'past_due' | 'canceled' | 'trialing'
  gateway: string
  current_period_start: string
  current_period_end: string
  trial_end?: string
  canceled_at?: string
  price_cents: number
  created_at: string
}

// -- Message 

export interface Message {
  id: string
  sender_profile_id: string
  receiver_profile_id: string
  body?: string
  is_ppv: boolean
  ppv_price_cents?: number
  is_locked: boolean
  read_at?: string
  created_at: string
  media: MediaFile[]
}

export interface Conversation {
  profile: PublicProfile
  last_message: Message
  unread_count: number
  is_online?: boolean
  last_seen_at?: string
}

// -- API helpers 

export interface PaginatedResponse<T> {
  data: T[]
  next_cursor: string | null
  has_more: boolean
}

export interface ApiError {
  error: {
    code: string
    message: string
    details?: Record<string, any>
  }
}

// -- Creator Analytics 

export interface CreatorStats {
  total_subscribers: number
  new_subscribers_this_month: number
  churned_this_month: number
  total_revenue_cents: number
  revenue_this_month_cents: number
  pending_withdrawal_cents: number
  total_posts: number
  total_views: number
}
