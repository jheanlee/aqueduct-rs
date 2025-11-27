import { Entity, PrimaryKey, Property } from '@mikro-orm/core';

@Entity()
export class IpBlacklist {
  @PrimaryKey()
  id!: number

  @Property({ type: 'inet' })
  subnet!: string

  @Property({ type: 'text' })
  comment!: string
}