import { Entity, PrimaryKey, Property, Unique } from '@mikro-orm/core';

@Entity()
export class User {
  @PrimaryKey({ length: 21 })
  id!: string

  @Property({ type: 'text' })
  @Unique()
  username!: string

  @Property({ type: 'text' })
  hashedPassword!: string

  //  8 random bytes, base64 encoded
  @Property({ length: 12 })
  salt!: string
}